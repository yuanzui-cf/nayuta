use crate::{
    core::{context::Project, post, schema::Schema},
    infra::{
        content::{Content, parse_yaml},
        error::{Result, user},
        fs::{self, Lock},
    },
};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path, str::FromStr};
use yaml_edit::{Document, Mapping, YamlFile};

#[derive(Clone)]
pub struct Frontmatter {
    pub content: Content,
    pub schema: Schema,
}

impl Frontmatter {
    pub fn open(project: &Project, path: &Path) -> Result<Self> {
        let path = fs::inside(&project.schema.posts_root, path)?;
        Ok(Self {
            content: Content::open(&path)?,
            schema: project.schema.post.clone(),
        })
    }
    pub fn save(&self, project: &Project, values: &mut Value) -> Result<()> {
        let _lock = Lock::acquire(&project.root)?;
        let current = Project::open(&project.root)?;
        if serde_json::to_value(&current.schema.post)? != serde_json::to_value(&self.schema)? {
            return Err(user("conflict", "Frontmatter schema"));
        }
        fs::inside(&project.schema.posts_root, &self.content.path)?;
        post::validate(
            &current,
            &post::site_config(&current, values)?,
            values,
            self.content.path.parent().unwrap(),
        )?;
        let source = self.render(values)?;
        fs::replace(
            &self.content.path,
            &fs::digest(self.content.source.as_bytes()),
            source.as_bytes(),
        )
    }
    pub fn render(&self, values: &Value) -> Result<String> {
        if values == &self.content.values {
            return Ok(self.content.source.clone());
        }
        let original = self.content.yaml().replace("\r\n", "\n");
        let parsed = YamlFile::parse(&original);
        if !parsed.errors().is_empty() {
            return Err(user("unsupported", parsed.errors().join("; ")));
        }
        let tree = parsed.tree();
        let doc = tree
            .documents()
            .next()
            .ok_or_else(|| user("unsupported", "YAML document"))?;
        let mapping = doc
            .as_mapping()
            .ok_or_else(|| user("unsupported", "YAML mapping"))?;
        patch(&mapping, &self.content.values, values)?;
        let mut yaml = tree.to_string();
        if !yaml.ends_with('\n') {
            yaml.push('\n');
        }
        // Refuse to persist a library transformation that changes any other metadata.
        if parse_yaml(&yaml)? != *values {
            return Err(user("unsupported", "YAML round trip"));
        }
        if self.content.newline == "\r\n" {
            yaml = yaml.replace('\n', "\r\n");
        }
        Ok(format!(
            "{}{}{}",
            &self.content.source[..self.content.start],
            yaml,
            &self.content.source[self.content.end..]
        ))
    }
}

fn patch(mapping: &Mapping, old: &Value, next: &Value) -> Result<()> {
    let keys: BTreeSet<_> = old
        .as_object()
        .into_iter()
        .flat_map(|m| m.keys())
        .chain(next.as_object().into_iter().flat_map(|m| m.keys()))
        .collect();
    for key in keys {
        if old.get(key) == next.get(key) {
            continue;
        }
        if let Some(value) = next.get(key) {
            if value.is_object()
                && old[key].is_object()
                && let Some(nested) = mapping
                    .get(key.as_str())
                    .and_then(|n| n.as_mapping().cloned())
            {
                patch(&nested, &old[key], value)?;
            } else {
                let raw = format!("value: {}\n", serde_json::to_string(value)?);
                let doc =
                    Document::from_str(&raw).map_err(|e| user("unsupported", e.to_string()))?;
                let node = doc
                    .get("value")
                    .ok_or_else(|| user("unsupported", "YAML value"))?;
                mapping.set(key.as_str(), node);
            }
        } else {
            mapping.remove(key.as_str());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::schema::Kind;
    use serde_json::json;
    #[test]
    fn comments_unknown_fields_bom_crlf_and_mdx_body_survive() {
        let source = "\u{feff}---\r\n# metadata\r\ntitle: Old # comment\r\npublishDate: '2026-10-04'\r\ncopyright:\r\n  enabled: true # nested\r\n  license: 'CC BY 4.0'\r\nunknown: [a, b] # untouched\r\n---\r\nimport X from './x.ts';\r\n\r\n<X />\r\n";
        let content = Content::parse(Path::new("index.mdx"), source.into()).unwrap();
        let editor = Frontmatter {
            content,
            schema: Schema::new(Kind::String),
        };
        let mut next = editor.content.values.clone();
        next["title"] = json!("新标题");
        next["copyright"]["enabled"] = json!(false);
        next["tags"] = json!(["Rust"]);
        let out = editor.render(&next).unwrap();
        assert!(out.starts_with("\u{feff}---\r\n# metadata\r\n"));
        assert!(out.contains("unknown: [a, b] # untouched\r\n"));
        assert!(out.contains("# comment") && out.contains("# nested"));
        let parsed = Content::parse(Path::new("index.mdx"), out).unwrap();
        assert_eq!(parsed.values, next);
        assert_eq!(&parsed.source[parsed.end..], &source[editor.content.end..]);
    }
    #[test]
    fn ambiguous_yaml_is_rejected() {
        for yaml in [
            "title: a\ntitle: b\n",
            "x: &a abc\ny: *a\n",
            "title: !!str abc\n",
            "x: {a: 1, a: 2}\n",
        ] {
            assert!(parse_yaml(yaml).is_err(), "{yaml}");
        }
    }
}
