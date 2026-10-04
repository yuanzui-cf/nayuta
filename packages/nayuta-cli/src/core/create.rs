use crate::{
    core::context::{Project, validate_config},
    infra::{
        config::schema::load,
        error::{Result, user},
        fs::{self, Lock},
        git::{self, template::Template},
    },
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub fn defaults(template: &Template, cwd: &Path) -> Result<Value> {
    let schema = load(&template.root)?;
    let mut config = schema.config.defaults();
    if !config.is_object() {
        config = json!({});
    }
    config["author"] = json!(git::identity(cwd).map(|i| i.0).unwrap_or_default());
    config["avatar"] = json!("@asset:src/content/assets/logo.svg");
    config["links"] = json!([]);
    Ok(config)
}

pub fn render_config(config: &Value) -> Result<String> {
    let object = config
        .as_object()
        .ok_or_else(|| user("validation", "Config object"))?;
    let mut text = String::from("import type { Config } from '@type/config';\n");
    let asset = object
        .get("avatar")
        .and_then(Value::as_str)
        .filter(|v| v.starts_with("@asset:"));
    if let Some(asset) = asset {
        let path = asset.trim_start_matches("@asset:");
        if !path.starts_with("src/content/assets/") || path.contains("..") || path.contains('\\') {
            return Err(user("validation", "avatar"));
        }
        text.push_str(&format!(
            "import logoUrl from {};\n",
            serde_json::to_string(&format!("./{}?url", path.trim_start_matches("src/")))?
        ));
    }
    text.push_str("\nconst config: Config = {\n");
    for (key, value) in object {
        let encoded = if key == "avatar" && asset.is_some() {
            "logoUrl".into()
        } else {
            serde_json::to_string(value)?
        };
        text.push_str(&format!(
            "  {}: {},\n",
            serde_json::to_string(key)?,
            encoded
        ));
    }
    text.push_str("};\n\nexport default config;\n");
    Ok(text)
}

pub fn create(
    template: &Template,
    target: &Path,
    config: &mut Value,
    cwd: &Path,
) -> Result<Project> {
    let schemas = load(&template.root)?;
    validate_config(&schemas, config, true)?;
    let identity = git::identity(cwd)?;
    let target = if target.is_absolute() {
        target.into()
    } else {
        cwd.join(target)
    };
    if target.exists() && std::fs::read_dir(&target)?.next().is_some() {
        return Err(user("exists", target.display().to_string()));
    }
    let parent = target
        .parent()
        .ok_or_else(|| user("validation", "target"))?;
    std::fs::create_dir_all(parent)?;
    let parent = parent.canonicalize()?;
    let target = parent.join(
        target
            .file_name()
            .ok_or_else(|| user("validation", "target"))?,
    );
    let _lock = Lock::acquire(&target)?;
    let existed = target.exists();
    if existed
        && (target.is_symlink() || !target.is_dir() || std::fs::read_dir(&target)?.next().is_some())
    {
        return Err(user("exists", target.display().to_string()));
    }
    let stage = tempfile::tempdir_in(&parent)?;
    copy_template(&template.root, stage.path())?;
    let posts_relative = schemas
        .posts_root
        .strip_prefix(&template.root)
        .map_err(|_| user("unsupported", "posts root"))?;
    for directory in ["src/content/assets", "src/content/pages"] {
        std::fs::create_dir_all(stage.path().join(directory))?;
    }
    std::fs::create_dir_all(stage.path().join(posts_relative))?;
    std::fs::write(
        stage.path().join("src/content/assets/logo.svg"),
        include_bytes!("../../assets/starter/logo.svg"),
    )?;
    std::fs::write(stage.path().join("src/content/pages/.gitkeep"), "")?;
    std::fs::write(stage.path().join(posts_relative).join(".gitkeep"), "")?;
    let title = config["title"].as_str().unwrap_or("Nayuta");
    std::fs::write(
        stage.path().join("src/content/index.md"),
        format!("---\ntitle: {}\n---\n", serde_json::to_string(title)?),
    )?;
    std::fs::write(stage.path().join("src/config.ts"), render_config(config)?)?;
    if let Some(asset) = config["avatar"]
        .as_str()
        .and_then(|s| s.strip_prefix("@asset:"))
    {
        fs::inside(stage.path(), &stage.path().join(asset))?;
    }
    let head = stage.path().join("src/layouts/head-base.astro");
    let source = std::fs::read_to_string(&head)?;
    // Adapt the known legacy PNG hint; versions already using a generic favicon need no patch.
    std::fs::write(
        &head,
        source.replace(
            "<link rel=\"icon\" type=\"image/png\" href={config.avatar} />",
            "<link rel=\"icon\" href={config.avatar} />",
        ),
    )?;
    std::fs::create_dir_all(stage.path().join(".nayuta"))?;
    std::fs::write(
        stage.path().join(".nayuta/project.json"),
        serde_json::to_string_pretty(
            &json!({"project":"nayuta","ref":template.reference,"commit":template.commit,"cliVersion":env!("CARGO_PKG_VERSION")}),
        )? + "\n",
    )?;
    Project::open(stage.path())?;
    let expected_snapshot = snapshot(stage.path())?;
    if existed {
        std::fs::remove_dir(&target)?;
    }
    std::fs::rename(stage.path(), &target)?;
    match git::initialize(&target, &identity).and_then(|_| Project::open(&target)) {
        Ok(project) => Ok(project),
        Err(error) => {
            // Preserve any concurrent user edits; only roll back our own unchanged files.
            if snapshot(&target)? == expected_snapshot {
                std::fs::remove_dir_all(&target)?;
                if existed {
                    std::fs::create_dir(&target)?;
                }
            } else {
                return Err(user("conflict", format!("{}; {}", target.display(), error)));
            }
            Err(error)
        }
    }
}

fn snapshot(root: &Path) -> Result<std::collections::BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = std::collections::BTreeMap::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git")
    {
        let entry = entry.map_err(|e| user("error", e.to_string()))?;
        if entry.file_type().is_symlink() {
            return Err(user("conflict", entry.path().display().to_string()));
        }
        if entry.file_type().is_dir() {
            files.insert(
                entry.path().strip_prefix(root).unwrap().into(),
                b"directory".to_vec(),
            );
        }
        if entry.file_type().is_file() {
            files.insert(
                entry.path().strip_prefix(root).unwrap().into(),
                fs::digest(&std::fs::read(entry.path())?),
            );
        }
    }
    Ok(files)
}

pub fn copy_template(source: &Path, target: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(source).into_iter().filter_entry(|e| {
        let relative = e.path().strip_prefix(source).unwrap();
        let first = relative
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy());
        !matches!(
            first.as_deref(),
            Some(
                ".git"
                    | "public"
                    | "packages"
                    | "docs"
                    | "tests"
                    | "node_modules"
                    | "dist"
                    | "target"
                    | ".astro"
                    | ".worktree"
                    | ".github"
                    | "scripts"
            )
        ) && !relative.starts_with("src/content")
    }) {
        let entry = entry.map_err(|e| user("error", e.to_string()))?;
        let relative = entry.path().strip_prefix(source).unwrap();
        if relative.as_os_str().is_empty() {
            continue;
        }
        if relative.components().count() == 1
            && !entry.file_type().is_dir()
            && ![
                "package.json",
                "bun.lock",
                "astro.config.ts",
                "tsconfig.json",
                "LICENSE",
                ".gitignore",
            ]
            .iter()
            .any(|f| relative == Path::new(f))
        {
            continue;
        }
        if entry.file_type().is_symlink() {
            return Err(user("unsupported", relative.display().to_string()));
        }
        let destination = target.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(destination)?;
        } else {
            std::fs::copy(entry.path(), destination)?;
        }
    }
    let package = target.join("package.json");
    let mut values: Value = serde_json::from_slice(&std::fs::read(&package)?)?;
    if let Some(object) = values.as_object_mut() {
        object.remove("workspaces");
    }
    if let Some(scripts) = values["scripts"].as_object_mut() {
        scripts
            .retain(|key, _| ["dev", "build", "preview", "astro", "check"].contains(&key.as_str()));
    }
    std::fs::write(package, serde_json::to_string_pretty(&values)? + "\n")?;
    Ok(())
}

pub fn target_path(cwd: &Path, path: &str) -> PathBuf {
    cwd.join(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_has_one_local_commit_and_no_demo_or_public() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let folder = tempfile::tempdir().unwrap();
        git::run(folder.path(), &["init", "--initial-branch=main"]).unwrap();
        git::run(folder.path(), &["config", "user.name", "Fixture"]).unwrap();
        git::run(
            folder.path(),
            &["config", "user.email", "fixture@example.org"],
        )
        .unwrap();
        let template = Template {
            root: source.clone(),
            reference: "test".into(),
            commit: "fixture".into(),
        };
        let mut config = json!({"title":"Personal","author":"Writer","avatar":"@asset:src/content/assets/logo.svg","site_url":"https://example.com"});
        let target = folder.path().join("blog");
        let project = create(&template, &target, &mut config, folder.path()).unwrap();
        assert!(!target.join("public").exists());
        assert!(!target.join("src/content/pages/demo").exists());
        assert!(target.join("src/content/assets/logo.svg").is_file());
        assert_eq!(
            git::run(&target, &["rev-list", "--all", "--count"]).unwrap(),
            "1"
        );
        assert_eq!(
            git::run(&target, &["show", "-s", "--format=%ae"]).unwrap(),
            git::identity(&target)
                .unwrap_or_else(|_| git::identity(folder.path()).unwrap())
                .1
        );
        assert_eq!(
            project.config().unwrap()["avatar"],
            json!("@asset:src/content/assets/logo.svg")
        );
        assert!(create(&template, &target, &mut config, &source).is_err());
        if std::env::var_os("NAYUTA_TEST_SITE_BUILD").is_some() {
            use crate::core::{frontmatter::Frontmatter, post, settings::Settings};
            let settings = Settings::open(&project).unwrap();
            let mut next = settings.values.clone();
            next["description"] = json!("Writer profile");
            settings.save(&project, &mut next).unwrap();
            let mut values = post::defaults(&project);
            values["title"] = json!("Hello, Nayuta");
            values["draft"] = json!(false);
            values["tags"] = json!([" Rust ", "Rust"]);
            let file = post::create(
                &project,
                &mut post::NewPost {
                    slug: "hello-nayuta".into(),
                    format: "mdx".into(),
                    date_prefix: false,
                    values,
                },
            )
            .unwrap();
            let editor = Frontmatter::open(&project, &file).unwrap();
            let mut values = editor.content.values.clone();
            values["description"] = json!("Edited with the same version schema");
            Frontmatter::save(&editor, &project, &mut values).unwrap();
            assert_eq!(
                git::run(&target, &["rev-list", "--all", "--count"]).unwrap(),
                "1"
            );
            for args in [
                ["install"].as_slice(),
                ["run", "check"].as_slice(),
                ["run", "build"].as_slice(),
            ] {
                assert!(
                    std::process::Command::new("bun")
                        .current_dir(&target)
                        .args(args)
                        .status()
                        .unwrap()
                        .success()
                );
            }
            let output = std::fs::read_to_string(target.join("dist/index.html")).unwrap();
            assert!(!output.contains("/assets/images/avatar.png"));
            assert!(output.contains("data:image/svg+xml") || output.contains("/_astro/logo"));
            let article =
                std::fs::read_to_string(target.join("dist/posts/hello-nayuta/index.html")).unwrap();
            assert!(article.contains("Hello, Nayuta") && article.contains("Rust"));
        }
    }
}
