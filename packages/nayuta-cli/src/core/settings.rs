use crate::{
    core::{
        context::{Project, static_config, validate_config},
        post,
        schema::{Kind, Schema},
    },
    infra::{
        config::typescript::{Module, Reader, kind, properties, unwrap},
        content::{self, Content},
        error::{Result, user},
        fs::{self, Lock},
    },
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct Settings {
    pub values: Value,
    pub readonly: Vec<String>,
    pub schema: Schema,
    path: PathBuf,
    module: Module,
    hash: Vec<u8>,
}

impl Settings {
    pub fn open(project: &Project) -> Result<Self> {
        let path = fs::inside(&project.root, &project.root.join("src/config.ts"))?;
        let mut reader = Reader::new(&project.root)?;
        let module = reader.module(&path)?;
        let (values, readonly) = static_config(&project.root)?;
        let mut keys = BTreeSet::new();
        if properties(module.config_object()?).any(|(key, _)| !keys.insert(key)) {
            return Err(user("unsupported", "duplicate config key"));
        }
        let hash = fs::digest(module.source.as_bytes());
        Ok(Self {
            values,
            readonly,
            schema: project.schema.config.clone(),
            path,
            module,
            hash,
        })
    }

    pub fn save(&self, project: &Project, next: &mut Value) -> Result<()> {
        let _lock = Lock::acquire(&project.root)?;
        let mut current = Project::open(&project.root)?;
        if serde_json::to_value(&current.schema.config)? != serde_json::to_value(&self.schema)? {
            return Err(user("conflict", "Config schema"));
        }
        if let Kind::Object(fields) = &mut current.schema.config.kind {
            fields.retain(|f| !self.readonly.contains(&f.name));
        }
        for key in &self.readonly {
            if next.get(key).is_some() {
                return Err(user("unsupported", format!("{key}: computed")));
            }
        }
        validate_config(&current.schema, next, false)?;
        if next.get("copyright") != self.values.get("copyright") {
            for path in content::paths(&current.schema.posts_root)? {
                let mut article = Content::open(&path)?;
                post::validate(&current, next, &mut article.values, path.parent().unwrap())?;
            }
        }
        let source = self.render(&project.root, next)?;
        Module::parse(&self.path, source.clone())?;
        fs::replace(&self.path, &self.hash, source.as_bytes())
    }

    fn render(&self, root: &Path, next: &Value) -> Result<String> {
        let mut edits = Vec::new();
        let next = next.clone();
        let mut asset = None;
        if next.get("avatar") != self.values.get("avatar")
            && let Some(path) = next["avatar"]
                .as_str()
                .and_then(|s| s.strip_prefix("@asset:"))
        {
            fs::inside(root, &root.join(path))?;
            if !path.starts_with("src/") || path.contains(['\\', '?', '#']) {
                return Err(user("validation", "avatar asset"));
            }
            let mut counter = 0;
            let identifier = loop {
                let identifier = format!("nayutaAvatar{counter}");
                if !self.module.source.contains(&identifier) {
                    break identifier;
                }
                counter += 1;
            };
            let specifier = serde_json::to_string(&format!("./{}?url", &path[4..]))?;
            asset = Some((identifier, specifier));
        }
        patch_object(
            &self.module,
            self.module.config_object()?,
            &self.values,
            &next,
            &self.readonly,
            asset.as_ref().map(|a| a.0.as_str()),
            &mut edits,
        )?;
        if let Some((identifier, specifier)) = asset {
            let offset = self
                .module
                .declarations()
                .filter(|n| kind(n) == "ImportDeclaration")
                .filter_map(|n| n["end"].as_u64())
                .max()
                .unwrap_or(if self.module.source.starts_with('\u{feff}') {
                    3
                } else {
                    0
                }) as usize;
            edits.push((
                offset,
                offset,
                format!("\nimport {identifier} from {specifier};\n"),
            ));
        }
        edits.reverse();
        edits.sort_by_key(|e| std::cmp::Reverse(e.0));
        let mut out = self.module.source.clone();
        for (start, end, text) in edits {
            out.replace_range(start..end, &text);
        }
        // Preserve the user's newline convention for newly inserted source as well.
        if self.module.source.contains("\r\n") {
            out = out.replace("\r\n", "\n").replace('\n', "\r\n");
        }
        Ok(out)
    }
}

type Edit = (usize, usize, String);
fn range(node: &Value) -> Result<(usize, usize)> {
    Ok((
        node["start"]
            .as_u64()
            .ok_or_else(|| user("unsupported", "AST span"))? as usize,
        node["end"]
            .as_u64()
            .ok_or_else(|| user("unsupported", "AST span"))? as usize,
    ))
}

fn patch_object(
    module: &Module,
    node: &Value,
    old: &Value,
    next: &Value,
    readonly: &[String],
    avatar: Option<&str>,
    edits: &mut Vec<Edit>,
) -> Result<()> {
    let node = unwrap(node);
    let members = node["properties"]
        .as_array()
        .ok_or_else(|| module.unsupported(node))?;
    let keys: BTreeSet<_> = old
        .as_object()
        .into_iter()
        .flat_map(|o| o.keys())
        .chain(next.as_object().into_iter().flat_map(|o| o.keys()))
        .collect();
    let mut insertions = Vec::new();
    for key in keys {
        if readonly.contains(key) || old.get(key) == next.get(key) {
            continue;
        }
        let property = members
            .iter()
            .find(|n| crate::infra::config::typescript::name(&n["key"]) == key);
        let encoded = next
            .get(key)
            .map(|v| {
                if key == "avatar"
                    && let Some(identifier) = avatar
                {
                    return Ok(identifier.to_owned());
                }
                serde_json::to_string(v)
            })
            .transpose()?;
        match (property, encoded) {
            (Some(member), Some(encoded)) => {
                let expression = unwrap(&member["value"]);
                if kind(expression) == "ObjectExpression"
                    && old[key].is_object()
                    && next[key].is_object()
                {
                    patch_object(module, expression, &old[key], &next[key], &[], None, edits)?;
                } else if member["shorthand"] == true {
                    let (start, end) = range(member)?;
                    edits.push((
                        start,
                        end,
                        format!("{}: {encoded}", serde_json::to_string(key)?),
                    ));
                } else {
                    let (start, end) = range(&member["value"])?;
                    edits.push((start, end, encoded));
                }
            }
            (Some(member), None) => {
                let (start, mut end) = range(member)?;
                let suffix = &module.source[end..];
                let whitespace = suffix.len() - suffix.trim_start().len();
                if suffix[whitespace..].starts_with(',') {
                    end += whitespace + 1;
                }
                edits.push((start, end, String::new()));
            }
            (None, Some(encoded)) => {
                insertions.push(format!("  {}: {encoded},", serde_json::to_string(key)?))
            }
            _ => {}
        }
    }
    if !insertions.is_empty() {
        if let Some(last) = members.last() {
            let (_, end) = range(last)?;
            if !module.source[end..range(node)?.1 - 1]
                .trim_start()
                .starts_with(',')
            {
                edits.push((end, end, ",".into()));
            }
        }
        let end = range(node)?.1 - 1;
        edits.push((end, end, format!("\n{}\n", insertions.join("\n"))));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn patches_preserve_comments_expressions_and_unknown_fields() {
        let source = "// header\nconst extra = true;\nconst config = {title: 'Old', // title comment\n copyright: {enabled: true, /* keep */ license: 'CC BY 4.0'}, unknown: 7, computed: Math.random()};\nexport default config;\n";
        let module = Module::parse(Path::new("config.ts"), source.into()).unwrap();
        let values =
            json!({"title":"Old","copyright":{"enabled":true,"license":"CC BY 4.0"},"unknown":7});
        let settings = Settings {
            values: values.clone(),
            readonly: vec!["computed".into()],
            schema: Schema::new(Kind::String),
            path: module.path.clone(),
            hash: fs::digest(source.as_bytes()),
            module,
        };
        let mut next = values;
        next["title"] = json!("新的标题");
        next["copyright"]["enabled"] = json!(false);
        next["description"] = json!("Hi");
        let edited = settings.render(Path::new("."), &next).unwrap();
        assert!(
            edited.contains("// header")
                && edited.contains("// title comment")
                && edited.contains("/* keep */")
        );
        assert!(edited.contains("computed: Math.random()") && edited.contains("unknown: 7"));
        Module::parse(Path::new("config.ts"), edited).unwrap();
    }
    #[test]
    fn stale_file_is_never_overwritten() {
        let folder = tempfile::tempdir().unwrap();
        let file = folder.path().join("config.ts");
        std::fs::write(&file, "old").unwrap();
        let hash = fs::digest(b"old");
        std::fs::write(&file, "external").unwrap();
        assert!(fs::replace(&file, &hash, b"new").is_err());
        assert_eq!(std::fs::read_to_string(file).unwrap(), "external");
    }
}
