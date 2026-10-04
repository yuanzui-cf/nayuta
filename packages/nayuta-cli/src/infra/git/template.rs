use super::{
    github::{REPOSITORY, cache_dir},
    output, run,
};
use crate::infra::error::{Result, read, user};
use serde_json::json;
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
    if let Ok(data) = read(&entry)
        && let Ok(record) = serde_json::from_str::<serde_json::Value>(&data)
        && let Some(commit) = record["commit"].as_str()
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
    let commit = sparse_fetch(
        stage.path(),
        &format!("https://github.com/{REPOSITORY}.git"),
        reference,
    )?;
    let destination = cache.join("templates").join(&commit);
    if !destination.exists() {
        std::fs::remove_dir_all(stage.path().join(".git"))?;
        std::fs::rename(stage.path(), &destination)?;
    }
    std::fs::create_dir_all(entry.parent().unwrap())?;
    let record = json!({"ref":reference,"commit":commit});
    std::fs::write(entry, serde_json::to_vec(&record)?)?;
    Ok(Template {
        root: destination,
        reference: reference.into(),
        commit,
    })
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
