use crate::infra::error::{Result, user};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use yaml_rust2::{
    Yaml, YamlLoader,
    scanner::{Scanner, TokenType},
};

#[derive(Clone)]
pub struct Content {
    pub path: PathBuf,
    pub source: String,
    pub start: usize,
    pub end: usize,
    pub values: Value,
    pub newline: &'static str,
}

impl Content {
    pub fn open(path: &Path) -> Result<Self> {
        Self::parse(path, std::fs::read_to_string(path)?)
    }
    pub fn parse(path: &Path, source: String) -> Result<Self> {
        let bom = if source.starts_with('\u{feff}') { 3 } else { 0 };
        let newline = if source[bom..].starts_with("---\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        if !source[bom..].starts_with(&format!("---{newline}")) {
            return Err(user(
                "validation",
                format!("{}: frontmatter", path.display()),
            ));
        }
        let start = bom + 3 + newline.len();
        let mut end = None;
        let mut offset = start;
        for line in source[start..].split_inclusive('\n') {
            if matches!(line.trim_end_matches(['\r', '\n']), "---" | "...") {
                end = Some(offset);
                break;
            }
            offset += line.len();
        }
        let end = end.ok_or_else(|| user("validation", "frontmatter delimiter"))?;
        let values = parse_yaml(&source[start..end])?;
        Ok(Self {
            path: path.into(),
            source,
            start,
            end,
            values,
            newline,
        })
    }
    pub fn yaml(&self) -> &str {
        &self.source[self.start..self.end]
    }
}

pub fn parse_yaml(text: &str) -> Result<Value> {
    // Do not silently resolve aliases/tags and then overwrite their authoring semantics.
    let mut scanner = Scanner::new(text.chars());
    for token in scanner.by_ref() {
        if matches!(
            token.1,
            TokenType::Anchor(_) | TokenType::Alias(_) | TokenType::Tag(_, _)
        ) {
            return Err(user("unsupported", "YAML anchor, alias or tag"));
        }
    }
    if let Some(error) = scanner.get_error() {
        return Err(user("validation", error.to_string()));
    }
    let docs = YamlLoader::load_from_str(text).map_err(|e| user("validation", e.to_string()))?;
    if docs.len() != 1 {
        return Err(user("validation", "one YAML document required"));
    }
    let value = yaml_value(&docs[0])?;
    if !value.is_object() {
        return Err(user("validation", "frontmatter mapping required"));
    }
    Ok(value)
}

fn yaml_value(yaml: &Yaml) -> Result<Value> {
    Ok(match yaml {
        Yaml::String(s) => json!(s),
        Yaml::Boolean(v) => json!(v),
        Yaml::Integer(v) => json!(v),
        Yaml::Real(v) => v
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(|| user("validation", "finite YAML number"))?,
        Yaml::Null => Value::Null,
        Yaml::Array(items) => Value::Array(items.iter().map(yaml_value).collect::<Result<_>>()?),
        Yaml::Hash(map) => Value::Object(
            map.iter()
                .map(|(k, v)| {
                    let key = k
                        .as_str()
                        .ok_or_else(|| user("unsupported", "YAML non-string key"))?;
                    if key == "<<" {
                        return Err(user("unsupported", "YAML merge"));
                    }
                    Ok((key.to_owned(), yaml_value(v)?))
                })
                .collect::<Result<_>>()?,
        ),
        _ => return Err(user("unsupported", "YAML value")),
    })
}

/// JSON flow values are valid YAML, and explicitly quote dates and special strings.
pub fn render(values: &Value) -> Result<String> {
    let mut out = String::new();
    for (key, value) in values
        .as_object()
        .ok_or_else(|| user("validation", "mapping"))?
    {
        out.push_str(&format!(
            "{}: {}\n",
            serde_json::to_string(key)?,
            serde_json::to_string(value)?
        ));
    }
    Ok(out)
}

pub fn paths(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().filter_entry(|e| {
        e.path() == root
            || (!e.file_name().to_string_lossy().starts_with('_') && e.file_name() != "assets")
    }) {
        let entry = entry.map_err(|e| user("error", e.to_string()))?;
        if entry.file_type().is_file()
            && matches!(
                entry.path().extension().and_then(|s| s.to_str()),
                Some("md" | "mdx")
            )
        {
            paths.push(entry.into_path());
        }
    }
    paths.sort();
    Ok(paths)
}
