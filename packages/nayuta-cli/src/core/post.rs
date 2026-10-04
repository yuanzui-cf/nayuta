use crate::{
    core::context::Project,
    infra::{
        content::{self, Content},
        error::{Result, user},
        fs::{Lock, inside},
    },
};
use chrono::{Local, NaiveDate};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

#[derive(Clone)]
pub struct NewPost {
    pub slug: String,
    pub format: String,
    pub date_prefix: bool,
    pub values: Value,
}

pub fn defaults(project: &Project) -> Value {
    let mut values = project.schema.post.defaults();
    if project.schema.post.field("publishDate").is_some() {
        values["publishDate"] = json!(Local::now().format("%Y-%m-%d").to_string());
    }
    if project.schema.post.field("draft").is_some() {
        values["draft"] = json!(true);
    }
    values
}

pub fn validate(
    project: &Project,
    config: &Value,
    values: &mut Value,
    directory: &Path,
) -> Result<()> {
    project.schema.post.normalize(values);
    project.schema.post.validate(values, "")?;
    if let Some(date) = values.get("publishDate").and_then(Value::as_str) {
        let valid = NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
            || chrono::DateTime::parse_from_rfc3339(date).is_ok();
        if !valid {
            return Err(user("validation", "publishDate: YYYY-MM-DD / RFC 3339"));
        }
    }
    if let Some(cover) = values.get("cover").and_then(Value::as_str) {
        if cover.starts_with("http://") || cover.starts_with("https://") {
            reqwest::Url::parse(cover).map_err(|_| user("validation", "cover"))?;
        } else if cover.starts_with('/') {
            inside(
                &project.root,
                &project
                    .root
                    .join("public")
                    .join(cover.trim_start_matches('/')),
            )?;
        } else {
            inside(&project.root, &directory.join(cover))?;
        }
    }
    let enabled = values["copyright"]["enabled"]
        .as_bool()
        .unwrap_or(config["copyright"]["enabled"] == true);
    if project.schema.post.field("copyright").is_some()
        && enabled
        && values["copyright"]["license"]
            .as_str()
            .or(config["copyright"]["license"].as_str())
            .is_none_or(|s| s.is_empty())
    {
        return Err(user("validation", "copyright.license: required"));
    }
    Ok(())
}

pub fn validate_slug(slug: &str) -> Result<()> {
    let stem = slug
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    if slug.is_empty()
        || slug.trim() != slug
        || slug.starts_with(['.', '_'])
        || slug.ends_with(['.', ' '])
        || slug.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
        || slug.chars().any(char::is_control)
        || slug.eq_ignore_ascii_case("assets")
        || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        || slug.len() > 160
        || github_slug(slug).is_empty()
    {
        return Err(user("validation", "slug"));
    }
    Ok(())
}

pub fn github_slug(text: &str) -> String {
    static REGEX: OnceLock<regress::Regex> = OnceLock::new();
    let regex = REGEX.get_or_init(|| {
        regress::Regex::new(include_str!("../../assets/github-slugger.txt").trim())
            .expect("vendored slug regex")
    });
    let utf16: Vec<_> = text.to_lowercase().encode_utf16().collect();
    let mut out = Vec::new();
    let mut cursor = 0;
    for m in regex.find_from_ucs2(&utf16, 0) {
        out.extend_from_slice(&utf16[cursor..m.range.start]);
        cursor = m.range.end;
    }
    out.extend_from_slice(&utf16[cursor..]);
    String::from_utf16_lossy(&out).replace(' ', "-")
}

pub fn id(root: &Path, path: &Path, values: &Value) -> Result<String> {
    if let Some(slug) = values.get("slug").and_then(Value::as_str) {
        return Ok(slug.to_owned());
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|_| user("validation", "post path"))?
        .with_extension("");
    let mut parts: Vec<_> = relative
        .components()
        .map(|c| github_slug(&c.as_os_str().to_string_lossy()))
        .collect();
    if parts.len() > 1 && parts.last().is_some_and(|s| s == "index") {
        parts.pop();
    }
    Ok(parts.join("/"))
}

pub fn create(project: &Project, options: &mut NewPost) -> Result<PathBuf> {
    validate_slug(&options.slug)?;
    if !matches!(options.format.as_str(), "md" | "mdx") {
        return Err(user("validation", "format: md / mdx"));
    }
    let _lock = Lock::acquire(&project.root)?;
    let mut name = options.slug.clone();
    if options.date_prefix {
        let date = options.values["publishDate"].as_str().unwrap_or_default();
        let date = date
            .get(..10)
            .ok_or_else(|| user("validation", "publishDate"))?;
        NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|_| user("validation", "publishDate"))?;
        name = format!("{date}-{name}");
    }
    let root = &project.schema.posts_root;
    std::fs::create_dir_all(root)?;
    inside(&project.root, root)?;
    let directory = root.join(name);
    let file = directory.join(format!("index.{}", options.format));
    if directory.exists() {
        return Err(user("exists", directory.display().to_string()));
    }
    validate(project, &project.config()?, &mut options.values, &directory)?;
    let next_id = id(root, &file, &options.values)?;
    for old in content::paths(root)? {
        let content = Content::open(&old)?;
        if id(root, &old, &content.values)? == next_id {
            return Err(user("exists", old.display().to_string()));
        }
    }
    let stage = tempfile::tempdir_in(root)?;
    std::fs::create_dir(stage.path().join("assets"))?;
    std::fs::write(
        stage.path().join(format!("index.{}", options.format)),
        format!("---\n{}---\n\n", content::render(&options.values)?),
    )?;
    std::fs::rename(stage.path(), &directory)?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn safe_unicode_slugs_and_astro_ids() {
        for slug in ["hello-world", "你好世界", "日本語"] {
            validate_slug(slug).unwrap();
        }
        for slug in [
            "../hi",
            "CON.txt",
            "aux",
            "LPT9",
            "assets",
            "_draft",
            "trailing.",
            "a/b",
            "😀",
        ] {
            assert!(validate_slug(slug).is_err(), "{slug}");
        }
        assert_eq!(github_slug("Hello, 世界! 😀"), "hello-世界-");
        assert_eq!(
            id(
                Path::new("/posts"),
                Path::new("/posts/Hello World/index.mdx"),
                &json!({})
            )
            .unwrap(),
            "hello-world"
        );
    }
    #[test]
    fn bundles_defaults_normalization_and_collisions() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let mut project = Project::open(&root).unwrap();
        let temp = tempfile::tempdir().unwrap();
        project.schema.posts_root = temp.path().join("posts");
        project.root = temp.path().into();
        std::fs::create_dir(temp.path().join("src")).unwrap();
        std::fs::write(temp.path().join("tsconfig.json"), "{}").unwrap();
        std::fs::write(temp.path().join("src/config.ts"), "export default {};").unwrap();
        let mut options = NewPost {
            slug: "Hello World".into(),
            format: "mdx".into(),
            date_prefix: true,
            values: defaults(&project),
        };
        options.values["title"] = json!("你好");
        options.values["tags"] = json!([" Rust ", "Rust"]);
        let file = create(&project, &mut options).unwrap();
        assert!(file.parent().unwrap().join("assets").is_dir());
        let parsed = Content::open(&file).unwrap();
        assert_eq!(parsed.values["draft"], true);
        assert_eq!(parsed.values["tags"], json!(["Rust"]));
        assert!(parsed.yaml().contains("\"publishDate\": \""));
        options.slug = "hello-world".into();
        options.format = "md".into();
        assert!(create(&project, &mut options).is_err());
    }
}
