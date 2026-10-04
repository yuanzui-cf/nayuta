use super::{
    github::{REPOSITORY, cache_dir},
    output, run,
};
use crate::infra::error::{Result, read, user};
use crate::infra::fs::Lock;
use serde_json::json;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Template {
    pub root: PathBuf,
    pub reference: String,
    pub commit: String,
}

pub fn fetch(reference: &str, offline: bool) -> Result<Template> {
    let cache = cache_dir();
    let key = format!("{:x}", sha2::Sha256::digest(reference.as_bytes()));
    use sha2::Digest;
    let entry = cache.join("refs").join(format!("{key}.json"));
    let _lock = Lock::acquire(&entry)?;
    if let Ok(data) = read(&entry)
        && let Ok(record) = serde_json::from_str::<serde_json::Value>(&data)
        && let Some(commit) = record["commit"].as_str()
        && commit.len() == 40
        && commit.bytes().all(|c| c.is_ascii_hexdigit())
        && cache
            .join("templates")
            .join(commit)
            .join("src/content.config.ts")
            .is_file()
        && offline
    {
        return Ok(Template {
            root: cache.join("templates").join(commit),
            reference: reference.into(),
            commit: commit.into(),
        });
    }
    if offline {
        return Err(user("not_found", format!("template: {reference}")));
    }
    std::fs::create_dir_all(cache.join("templates"))?;
    let stage = tempfile::tempdir_in(cache.join("templates"))?;
    let commit = match release_template(stage.path(), reference)? {
        Some(commit) => commit,
        None => sparse_fetch(
            stage.path(),
            &format!("https://github.com/{REPOSITORY}.git"),
            reference,
        )?,
    };
    crate::infra::config::schema::load(stage.path())?;
    let destination = cache.join("templates").join(&commit);
    if !destination.exists() {
        if stage.path().join(".git").exists() {
            std::fs::remove_dir_all(stage.path().join(".git"))?;
        }
        std::fs::rename(stage.path(), &destination)?;
    }
    std::fs::create_dir_all(entry.parent().unwrap())?;
    let record = json!({"ref":reference,"commit":commit});
    let mut temp = tempfile::NamedTempFile::new_in(entry.parent().unwrap())?;
    temp.write_all(&serde_json::to_vec(&record)?)?;
    temp.persist(entry).map_err(|e| e.error)?;
    Ok(Template {
        root: destination,
        reference: reference.into(),
        commit,
    })
}

fn download(url: &str, limit: u64) -> Result<Vec<u8>> {
    let parsed = reqwest::Url::parse(url).map_err(|_| user("network", "asset URL"))?;
    if parsed.scheme() != "https"
        || !matches!(
            parsed.host_str(),
            Some(
                "github.com"
                    | "objects.githubusercontent.com"
                    | "release-assets.githubusercontent.com"
            )
        )
    {
        return Err(user("network", "release asset origin"));
    }
    // Use a separate client so authenticated API headers never reach release storage.
    let client = reqwest::blocking::Client::builder()
        .user_agent("nayuta-cli")
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    let mut bytes = Vec::new();
    client
        .get(parsed)
        .send()?
        .error_for_status()?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(user("network", "release asset size"));
    }
    Ok(bytes)
}

fn release_template(target: &Path, reference: &str) -> Result<Option<String>> {
    let Ok(release) = super::github::release(reference) else {
        return Ok(None);
    };
    let filename = format!(
        "nayuta-template-{}.tar.gz",
        reference.trim_start_matches('v')
    );
    let Some(asset) = release.assets.iter().find(|a| a.name == filename) else {
        return Ok(None);
    };
    let checksum = release
        .assets
        .iter()
        .find(|a| a.name == format!("{filename}.sha256"))
        .ok_or_else(|| user("network", "template checksum missing"))?;
    let expected = String::from_utf8(download(&checksum.browser_download_url, 4096)?)
        .map_err(|_| user("network", "template checksum"))?;
    let expected = expected
        .split_whitespace()
        .next()
        .ok_or_else(|| user("network", "template checksum"))?;
    let bytes = download(&asset.browser_download_url, 64 * 1024 * 1024)?;
    unpack_template(target, &bytes, expected)?;
    let record: serde_json::Value =
        serde_json::from_str(&read(&target.join("nayuta-template.json"))?)?;
    let commit = record["commit"]
        .as_str()
        .filter(|s| s.len() == 40 && s.bytes().all(|c| c.is_ascii_hexdigit()))
        .ok_or_else(|| user("unsupported", "template commit"))?;
    if record["version"] != reference.trim_start_matches('v') {
        return Err(user("unsupported", "template version"));
    }
    std::fs::remove_file(target.join("nayuta-template.json"))?;
    Ok(Some(commit.into()))
}

fn unpack_template(target: &Path, bytes: &[u8], expected: &str) -> Result<()> {
    use sha2::Digest;
    if format!("{:x}", sha2::Sha256::digest(bytes)) != expected {
        return Err(user("network", "template SHA-256 mismatch"));
    }
    let decoder = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder);
    let mut total = 0;
    for entry in archive.entries()? {
        let mut entry = entry?;
        total += entry.size();
        if total > 256 * 1024 * 1024 {
            return Err(user("unsupported", "template unpack size"));
        }
        let path = entry.path()?.into_owned();
        let path: PathBuf = path
            .components()
            .filter(|c| !matches!(c, std::path::Component::CurDir))
            .collect();
        let kind = entry.header().entry_type();
        if path
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
            || !(kind.is_file() || kind.is_dir())
        {
            return Err(user("unsupported", "template archive path / link"));
        }
        if path.as_os_str().is_empty() {
            continue;
        }
        let allowed = path.starts_with("src") && !path.starts_with("src/content")
            || [
                "package.json",
                "bun.lock",
                "astro.config.ts",
                "tsconfig.json",
                "LICENSE",
                ".gitignore",
                "nayuta-template.json",
            ]
            .iter()
            .any(|p| path == Path::new(p));
        if !allowed {
            return Err(user("unsupported", path.display().to_string()));
        }
        if !entry.unpack_in(target)? {
            return Err(user("unsupported", "template archive path"));
        }
    }
    Ok(())
}

pub fn sparse_fetch(target: &Path, remote: &str, reference: &str) -> Result<String> {
    if reference.is_empty() || reference.starts_with('-') {
        return Err(user("not_found", reference));
    }
    run(
        target,
        &["check-ref-format", &format!("refs/heads/{reference}")],
    )
    .map_err(|_| user("not_found", reference))?;
    let refs = run(
        target,
        &[
            "ls-remote",
            "--exit-code",
            remote,
            &format!("refs/tags/{reference}"),
            &format!("refs/heads/{reference}"),
        ],
    )
    .map_err(|_| user("not_found", reference))?;
    let tag = format!("refs/tags/{reference}");
    let selected = refs
        .lines()
        .find(|line| line.ends_with(&tag))
        .or_else(|| refs.lines().next())
        .ok_or_else(|| user("not_found", reference))?;
    let remote_ref = selected
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| user("not_found", reference))?;
    run(target, &["init", "--initial-branch=main"])?;
    run(target, &["remote", "add", "origin", remote])?;
    run(target, &["config", "remote.origin.promisor", "true"])?;
    run(
        target,
        &["config", "remote.origin.partialclonefilter", "blob:none"],
    )?;
    let fetched = output(
        target,
        &[
            "fetch",
            "--depth=1",
            "--filter=blob:none",
            "origin",
            remote_ref,
        ],
    )?;
    if String::from_utf8_lossy(&fetched.stderr).contains("filtering not recognized") {
        return Err(user("unsupported", "Git blob filtering"));
    }
    let commit = run(target, &["rev-parse", "FETCH_HEAD^{commit}"])?;
    run(target, &["config", "core.sparseCheckout", "true"])?;
    run(target, &["config", "core.sparseCheckoutCone", "false"])?;
    std::fs::write(
        target.join(".git/info/sparse-checkout"),
        "/package.json\n/bun.lock\n/astro.config.ts\n/tsconfig.json\n/LICENSE\n/.gitignore\n/src/\n!/src/content/\n",
    )?;
    run(target, &["checkout", "--detach", &commit])?;
    Ok(commit)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verified_archives_reject_demo_public_and_bad_checksums() {
        use sha2::Digest;
        fn archive(path: &str) -> Vec<u8> {
            let gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            let mut tar = tar::Builder::new(gzip);
            let bytes = b"fixture";
            let mut header = tar::Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, path, &bytes[..]).unwrap();
            tar.into_inner().unwrap().finish().unwrap()
        }
        for path in [
            "src/config.ts",
            "src/content/posts/demo.md",
            "public/avatar.png",
        ] {
            let folder = tempfile::tempdir().unwrap();
            let bytes = archive(path);
            let hash = format!("{:x}", sha2::Sha256::digest(&bytes));
            assert_eq!(
                unpack_template(folder.path(), &bytes, &hash).is_ok(),
                path == "src/config.ts"
            );
            assert!(unpack_template(folder.path(), &bytes, "bad").is_err());
        }
    }
    #[test]
    fn sparse_checkout_does_not_fetch_content_or_public_blobs() {
        let upstream = tempfile::tempdir().unwrap();
        run(upstream.path(), &["init", "--initial-branch=main"]).unwrap();
        std::fs::create_dir_all(upstream.path().join("src/content/posts")).unwrap();
        std::fs::create_dir_all(upstream.path().join("public")).unwrap();
        std::fs::write(
            upstream.path().join("src/content/posts/demo.md"),
            "demo content",
        )
        .unwrap();
        std::fs::write(upstream.path().join("public/demo.svg"), "demo public").unwrap();
        std::fs::write(
            upstream.path().join("src/content.config.ts"),
            "export const collections={};",
        )
        .unwrap();
        run(upstream.path(), &["add", "."]).unwrap();
        run(
            upstream.path(),
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.org",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-m",
                "fixture",
            ],
        )
        .unwrap();
        run(
            upstream.path(),
            &["config", "uploadpack.allowFilter", "true"],
        )
        .unwrap();
        run(
            upstream.path(),
            &["config", "uploadpack.allowAnySHA1InWant", "true"],
        )
        .unwrap();
        let checkout = tempfile::tempdir().unwrap();
        let remote = format!("file://{}", upstream.path().display());
        sparse_fetch(checkout.path(), &remote, "main").unwrap();
        assert!(!checkout.path().join("src/content").exists());
        assert!(!checkout.path().join("public").exists());
        let missing = run(
            checkout.path(),
            &["rev-list", "--objects", "--all", "--missing=print"],
        )
        .unwrap();
        for source in ["src/content/posts/demo.md", "public/demo.svg"] {
            let blob = run(upstream.path(), &["rev-parse", &format!("HEAD:{source}")]).unwrap();
            assert!(missing.contains(&format!("?{blob}")));
        }
    }
}
