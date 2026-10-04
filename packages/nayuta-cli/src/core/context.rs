use crate::infra::{
    config::{
        schema::{ProjectSchema, load},
        typescript::{Reader, kind, name, properties},
    },
    error::{Result, read, user},
};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Project {
    pub root: PathBuf,
    pub version: String,
    pub schema: ProjectSchema,
}

impl Project {
    pub fn open(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let package: Value = serde_json::from_str(&read(&root.join("package.json"))?)?;
        let version = package["version"]
            .as_str()
            .ok_or_else(|| user("unsupported", "package.version"))?
            .into();
        let schema = load(&root)?;
        Ok(Self {
            root,
            version,
            schema,
        })
    }

    pub fn config(&self) -> Result<Value> {
        let mut reader = Reader::new(&self.root)?;
        let module = reader.module(&self.root.join("src/config.ts"))?;
        reader.value(&module, module.config_object()?, 0)
    }
}

pub fn discover(cwd: &Path) -> Result<Option<Project>> {
    let mut current = cwd.canonicalize()?;
    loop {
        if current.join("src/types/config.ts").is_file()
            && current.join("src/content.config.ts").is_file()
            && current.join("src/layouts/frame.astro").is_file()
            && current
                .join("src/assets/styles/themes/nayuta.css")
                .is_file()
        {
            let mut reader = Reader::new(&current)?;
            let module = reader.module(&current.join("src/types/config.ts"))?;
            if module
                .declaration("Config")
                .is_some_and(|n| kind(n) == "TSInterfaceDeclaration")
                && current.join("src/config.ts").is_file()
            {
                return Project::open(&current).map(Some);
            }
        }
        if !current.pop() {
            return Ok(None);
        }
    }
}

pub fn validate_config(schema: &ProjectSchema, values: &mut Value, creating: bool) -> Result<()> {
    schema.config.normalize(values);
    schema.config.validate(values, "")?;
    if schema.config.field("site_url").is_some() && values.get("site_url").is_some() {
        let url = values["site_url"]
            .as_str()
            .and_then(|s| reqwest::Url::parse(s).ok());
        if !url.is_some_and(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some()) {
            return Err(user("validation", "site_url: HTTP(S) URL"));
        }
    }
    if let Some(page_size) = values.get("postsPerPage")
        && page_size.as_u64().is_none_or(|n| n == 0)
    {
        return Err(user("validation", "postsPerPage: positive integer"));
    }
    if creating
        && values["copyright"]["enabled"] == true
        && !values["copyright"]["license"].is_string()
    {
        return Err(user("validation", "copyright.license: required"));
    }
    Ok(())
}

pub fn static_config(root: &Path) -> Result<(Value, Vec<String>)> {
    let mut reader = Reader::new(root)?;
    let module = reader.module(&root.join("src/config.ts"))?;
    let mut values = serde_json::Map::new();
    let mut readonly = Vec::new();
    for (key, expression) in properties(module.config_object()?) {
        match reader.value(&module, expression, 0) {
            Ok(value) => {
                values.insert(key.into(), value);
            }
            Err(_) => readonly.push(key.into()),
        }
    }
    if module.config_object()?["properties"]
        .as_array()
        .is_some_and(|p| p.iter().any(|p| kind(p) != "Property"))
    {
        return Err(user("unsupported", "config: spread or computed property"));
    }
    Ok((Value::Object(values), readonly))
}

pub fn config_export_name(root: &Path) -> Result<String> {
    let mut reader = Reader::new(root)?;
    let module = reader.module(&root.join("src/config.ts"))?;
    let object = module.config_object()?;
    Ok(name(object).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nearest_project_is_found_from_deep_directories() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        assert_eq!(
            discover(&root.join("src/assets/styles/themes"))
                .unwrap()
                .unwrap()
                .root,
            root
        );
        let empty = tempfile::tempdir().unwrap();
        assert!(discover(empty.path()).unwrap().is_none());
    }
}
