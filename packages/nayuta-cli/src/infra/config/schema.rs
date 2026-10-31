use super::typescript::{Module, Reader, kind, name, properties, property, unwrap};
use crate::{
    core::schema::{Field, Kind, Schema},
    infra::error::{Result, user},
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct ProjectSchema {
    pub config: Schema,
    pub post: Schema,
    pub posts_root: PathBuf,
    pub themes: Vec<String>,
}

impl Reader {
    pub fn type_schema(&mut self, module: &Module, node: &Value, depth: usize) -> Result<Schema> {
        if depth > 32 {
            return Err(module.unsupported(node));
        }
        let schema = match kind(node) {
            "TSStringKeyword" => Schema::new(Kind::String),
            "TSNumberKeyword" => Schema::new(Kind::Number),
            "TSBooleanKeyword" => Schema::new(Kind::Boolean),
            "TSLiteralType" => {
                Schema::new(Kind::Literal(self.value(module, &node["literal"], 0)?))
            }
            "TSArrayType" => Schema::new(Kind::Array(Box::new(self.type_schema(
                module,
                &node["elementType"],
                depth + 1,
            )?))),
            "TSUnionType" => Schema::new(Kind::Union(
                node["types"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| self.type_schema(module, n, depth + 1))
                    .collect::<Result<_>>()?,
            )),
            "TSTypeLiteral" | "TSInterfaceBody" => {
                let mut fields = Vec::new();
                for property in node["members"]
                    .as_array()
                    .or_else(|| node["body"].as_array())
                    .into_iter()
                    .flatten()
                {
                    if kind(property) != "TSPropertySignature"
                        || property["computed"].as_bool() == Some(true)
                    {
                        return Err(module.unsupported(property));
                    }
                    let mut schema = self.type_schema(
                        module,
                        &property["typeAnnotation"]["typeAnnotation"],
                        depth + 1,
                    )?;
                    schema.optional = property["optional"].as_bool().unwrap_or(false);
                    fields.push(Field {
                        name: name(&property["key"]).into(),
                        schema,
                    });
                }
                Schema::new(Kind::Object(fields))
            }
            "TSTypeReference" => {
                let key = name(&node["typeName"]);
                if key == "Date" {
                    return Ok(Schema::new(Kind::String));
                }
                if let Some(declaration) = module.declaration(key) {
                    self.declaration_schema(module, declaration, depth + 1)?
                } else {
                    let (imported, key) = self.imported(module, key)?;
                    self.declaration_schema(
                        &imported,
                        imported
                            .declaration(&key)
                            .ok_or_else(|| imported.unsupported(node))?,
                        depth + 1,
                    )?
                }
            }
            "TSIndexedAccessType"
                if kind(unparenthesized(&node["objectType"])) == "TSTypeQuery"
                    && kind(&node["indexType"]) == "TSNumberKeyword" =>
            {
                let expression = json!({"type":"Identifier", "name":name(&unparenthesized(&node["objectType"])["exprName"])});
                let values = self.value(module, &expression, 0)?;
                let values = values.as_array().ok_or_else(|| module.unsupported(node))?;
                Schema::new(Kind::Union(
                    values
                        .iter()
                        .map(|v| Schema::new(Kind::Literal(v.clone())))
                        .collect(),
                ))
            }
            "TSParenthesizedType" => {
                self.type_schema(module, &node["typeAnnotation"], depth + 1)?
            }
            _ => return Err(module.unsupported(node)),
        };
        Ok(schema)
    }

    fn declaration_schema(
        &mut self,
        module: &Module,
        node: &Value,
        depth: usize,
    ) -> Result<Schema> {
        match kind(node) {
            "TSTypeAliasDeclaration" => {
                self.type_schema(module, &node["typeAnnotation"], depth + 1)
            }
            "TSInterfaceDeclaration" => {
                let mut schema = self.type_schema(module, &node["body"], depth + 1)?;
                for extends in node["extends"].as_array().into_iter().flatten() {
                    let reference =
                        json!({"type":"TSTypeReference","typeName":extends["expression"]});
                    let parent = self.type_schema(module, &reference, depth + 1)?;
                    if let Kind::Object(fields) = &mut schema.kind {
                        for field in parent.fields() {
                            if !fields.iter().any(|f| f.name == field.name) {
                                fields.push(field.clone());
                            }
                        }
                    }
                }
                Ok(schema)
            }
            _ => Err(module.unsupported(node)),
        }
    }

    pub fn zod_schema(
        &mut self,
        module: &Module,
        expression: &Value,
        depth: usize,
    ) -> Result<Schema> {
        if depth > 48 {
            return Err(module.unsupported(expression));
        }
        let node = unwrap(expression);
        if kind(node) == "Identifier" {
            if let Some(expression) = module.binding(name(node)) {
                return self.zod_schema(module, expression, depth + 1);
            }
            let (imported, key) = self.imported(module, name(node))?;
            return self.zod_schema(
                &imported,
                imported
                    .binding(&key)
                    .ok_or_else(|| imported.unsupported(node))?,
                depth + 1,
            );
        }
        if kind(node) == "ArrowFunctionExpression" {
            if node["expression"].as_bool() == Some(true) {
                return self.zod_schema(module, &node["body"], depth + 1);
            }
            let body = node["body"]["body"]
                .as_array()
                .ok_or_else(|| module.unsupported(node))?;
            if body.len() == 1 && kind(&body[0]) == "ReturnStatement" {
                return self.zod_schema(module, &body[0]["argument"], depth + 1);
            }
            return Err(module.unsupported(node));
        }
        if kind(node) != "CallExpression" {
            return Err(module.unsupported(node));
        }
        let callee = &node["callee"];
        if name(callee) == "image" {
            return Ok(Schema::new(Kind::Image));
        }
        if kind(callee) != "MemberExpression" || callee["computed"].as_bool() == Some(true) {
            return Err(module.unsupported(node));
        }
        let method = name(&callee["property"]);
        let receiver = &callee["object"];
        let args = node["arguments"]
            .as_array()
            .ok_or_else(|| module.unsupported(node))?;
        let arg = args.first().unwrap_or(&Value::Null);
        let base_zod = kind(receiver) == "Identifier"
            && module
                .import(name(receiver))
                .is_some_and(|(source, imported)| source == "astro/zod" && imported == "z");
        if base_zod {
            return match method {
                "string" => Ok(Schema::new(Kind::String)),
                "number" => Ok(Schema::new(Kind::Number)),
                "boolean" => Ok(Schema::new(Kind::Boolean)),
                "object" | "looseObject" => {
                    let shape = args.first().ok_or_else(|| module.unsupported(node))?;
                    if kind(shape) != "ObjectExpression"
                        || shape["properties"].as_array().is_some_and(|p| {
                            p.iter().any(|n| {
                                kind(n) != "Property" || n["computed"].as_bool() == Some(true)
                            })
                        })
                    {
                        return Err(module.unsupported(shape));
                    }
                    Ok(Schema::new(Kind::Object(
                        properties(shape)
                            .map(|(key, value)| {
                                Ok(Field {
                                    name: key.into(),
                                    schema: self.zod_schema(module, value, depth + 1)?,
                                })
                            })
                            .collect::<Result<_>>()?,
                    )))
                }
                "array" => Ok(Schema::new(Kind::Array(Box::new(self.zod_schema(
                    module,
                    arg,
                    depth + 1,
                )?)))),
                "union" => Ok(Schema::new(Kind::Union(
                    arg["elements"]
                        .as_array()
                        .ok_or_else(|| module.unsupported(node))?
                        .iter()
                        .map(|n| self.zod_schema(module, n, depth + 1))
                        .collect::<Result<_>>()?,
                ))),
                "enum" => {
                    let values = self.value(module, arg, 0)?;
                    Ok(Schema::new(Kind::Union(
                        values
                            .as_array()
                            .ok_or_else(|| module.unsupported(node))?
                            .iter()
                            .map(|v| Schema::new(Kind::Literal(v.clone())))
                            .collect(),
                    )))
                }
                "literal" => Ok(Schema::new(Kind::Literal(self.value(module, arg, 0)?))),
                _ => Err(module.unsupported(node)),
            };
        }
        let mut schema = self.zod_schema(module, receiver, depth + 1)?;
        match method {
            "optional" => schema.optional = true,
            "nullable" => {
                schema.kind = Kind::Union(vec![
                    schema.clone(),
                    Schema::new(Kind::Literal(Value::Null)),
                ])
            }
            "default" => schema.default = Some(self.value(module, arg, 0)?),
            "trim" => schema.trim = true,
            "min" => schema.min = self.value(module, arg, 0)?.as_f64(),
            "max" => schema.max = self.value(module, arg, 0)?.as_f64(),
            "int" => schema.integer = true,
            "positive" => schema.min = Some(f64::MIN_POSITIVE),
            "regex" => {
                schema.pattern = Some((
                    arg["regex"]["pattern"]
                        .as_str()
                        .ok_or_else(|| module.unsupported(node))?
                        .into(),
                    arg["regex"]["flags"].as_str().unwrap_or_default().into(),
                ))
            }
            "transform" if is_set_dedup(arg) => schema.unique = true,
            _ => return Err(module.unsupported(node)),
        }
        Ok(schema)
    }
}

fn is_set_dedup(node: &Value) -> bool {
    let body = &node["body"];
    let elements = body["elements"].as_array();
    kind(node) == "ArrowFunctionExpression"
        && kind(body) == "ArrayExpression"
        && elements.is_some_and(|a| a.len() == 1)
        && kind(&body["elements"][0]) == "SpreadElement"
        && kind(&body["elements"][0]["argument"]) == "NewExpression"
        && name(&body["elements"][0]["argument"]["callee"]) == "Set"
        && body["elements"][0]["argument"]["arguments"]
            .as_array()
            .is_some_and(|a| a.len() == 1)
        && name(&body["elements"][0]["argument"]["arguments"][0]) == name(&node["params"][0])
}

fn unparenthesized(node: &Value) -> &Value {
    if kind(node) == "TSParenthesizedType" {
        unparenthesized(&node["typeAnnotation"])
    } else {
        node
    }
}

pub fn load(root: &Path) -> Result<ProjectSchema> {
    let mut reader = Reader::new(root)?;
    let types = reader.module(&root.join("src/types/config.ts"))?;
    let mut config = reader.declaration_schema(
        &types,
        types
            .declaration("Config")
            .ok_or_else(|| user("unsupported", "Config"))?,
        0,
    )?;
    runtime_config(&mut reader, root, &mut config)?;
    let content = reader.module(&root.join("src/content.config.ts"))?;
    let collections = content
        .binding("collections")
        .ok_or_else(|| user("unsupported", "collections"))?;
    let post_ref =
        property(collections, "posts").ok_or_else(|| user("unsupported", "collections.posts"))?;
    let collection = content
        .binding(name(post_ref))
        .ok_or_else(|| content.unsupported(post_ref))?;
    let options = &collection["arguments"][0];
    let post = reader.zod_schema(
        &content,
        property(options, "schema").ok_or_else(|| content.unsupported(options))?,
        0,
    )?;
    let loader = property(options, "loader").ok_or_else(|| content.unsupported(options))?;
    let loader_options = &loader["arguments"][0];
    let pattern = property(loader_options, "pattern").ok_or_else(|| content.unsupported(loader))?;
    if reader.value(&content, pattern, 0)?
        != json!(["**/[^_]*.{md,mdx}", "!**/assets/**", "!**/_*/**"])
        || property(loader_options, "generateId").is_some()
    {
        return Err(user("unsupported", "posts loader pattern / generateId"));
    }
    let base =
        property(&loader["arguments"][0], "base").ok_or_else(|| content.unsupported(loader))?;
    let base = reader.value(&content, base, 0)?;
    let relative = base
        .as_str()
        .ok_or_else(|| content.unsupported(loader))?
        .trim_start_matches("./");
    let posts_root = root.join(relative);
    if !posts_root.starts_with(root)
        || Path::new(relative).components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
    {
        return Err(user("unsupported", relative));
    }
    let mut themes: Vec<_> = std::fs::read_dir(root.join("src/assets/styles/themes"))?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension()?.to_str()? == "css")
                .then(|| path.file_stem().unwrap().to_string_lossy().into_owned())
        })
        .collect();
    themes.sort();
    Ok(ProjectSchema {
        config,
        post,
        posts_root,
        themes,
    })
}

/// Infer literal defaults and the required site URL from the actual theme consumers.
/// These are AST adapters for the same code used by Astro, not another schema source.
fn runtime_config(reader: &mut Reader, root: &Path, config: &mut Schema) -> Result<()> {
    for path in [
        "astro.config.ts",
        "src/layouts/head-base.astro",
        "src/layouts/widgets/post-list/PostList.astro",
        "src/pages/posts/[slug].astro",
    ] {
        let file = root.join(path);
        if !file.is_file() {
            continue;
        }
        let mut source = crate::infra::error::read(&file)?;
        if path.ends_with(".astro") {
            source = source
                .strip_prefix("---")
                .and_then(|s| s.split_once("---").map(|(script, _)| script.to_owned()))
                .ok_or_else(|| user("unsupported", format!("{path}: Astro script")))?;
        }
        let module = Module::parse(&file, source)?;
        if path == "astro.config.ts"
            && let Some(binding) = module.binding("site")
            && config_path(binding).as_deref() == Some("site_url")
        {
            let required = module.declarations().any(|node| {
                kind(node) == "IfStatement"
                    && kind(&node["test"]) == "UnaryExpression"
                    && node["test"]["operator"] == "!"
                    && name(&node["test"]["argument"]) == "site"
                    && node["consequent"]["body"]
                        .as_array()
                        .is_some_and(|body| body.iter().any(|n| kind(n) == "ThrowStatement"))
            });
            if required && let Some(field) = config.at_mut("site_url") {
                field.optional = false;
            }
        }
        infer_defaults(reader, &module, &module.ast, config)?;
    }
    Ok(())
}
fn config_path(node: &Value) -> Option<String> {
    let node = if kind(node) == "ChainExpression" {
        &node["expression"]
    } else {
        unwrap(node)
    };
    if kind(node) == "Identifier" && name(node) == "config" {
        return Some(String::new());
    }
    if kind(node) != "MemberExpression" || node["computed"] == true {
        return None;
    }
    let prefix = config_path(&node["object"])?;
    Some(if prefix.is_empty() {
        name(&node["property"]).into()
    } else {
        format!("{prefix}.{}", name(&node["property"]))
    })
}
fn config_paths(node: &Value, paths: &mut Vec<String>) {
    if let Some(path) = config_path(node)
        && !path.is_empty()
    {
        paths.push(path);
        return;
    }
    if let Some(object) = node.as_object() {
        for value in object.values() {
            config_paths(value, paths);
        }
    }
    if let Some(array) = node.as_array() {
        for value in array {
            config_paths(value, paths);
        }
    }
}
fn infer_defaults(
    reader: &mut Reader,
    module: &Module,
    node: &Value,
    config: &mut Schema,
) -> Result<()> {
    if kind(node) == "LogicalExpression"
        && matches!(node["operator"].as_str(), Some("??" | "||"))
        && kind(&node["right"]) == "Literal"
    {
        let mut paths = Vec::new();
        config_paths(&node["left"], &mut paths);
        for path in paths {
            if let Some(field) = config.at_mut(&path) {
                field.default = Some(reader.value(module, &node["right"], 0)?);
            }
        }
    }
    if let Some(object) = node.as_object() {
        for value in object.values() {
            infer_defaults(reader, module, value, config)?;
        }
    }
    if let Some(array) = node.as_array() {
        for value in array {
            infer_defaults(reader, module, value, config)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_repository_types_and_zod_are_read() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let schemas = load(&root).unwrap();
        assert!(schemas.config.field("links").is_some());
        assert!(!schemas.config.field("site_url").unwrap().optional);
        assert_eq!(
            schemas.config.field("theme").unwrap().default,
            Some(json!("nayuta-aqua"))
        );
        assert_eq!(
            schemas.config.field("postsPerPage").unwrap().default,
            Some(json!(10))
        );
        assert_eq!(
            schemas.config.at("copyright.enabled").unwrap().default,
            Some(json!(false))
        );
        assert_eq!(
            schemas.post.field("draft").unwrap().default,
            Some(json!(false))
        );
        assert!(schemas.post.field("exclude_in_search").is_some());
        assert_eq!(
            schemas
                .config
                .at("copyright.license")
                .unwrap()
                .enum_values()
                .unwrap()
                .len(),
            6
        );
        let mut post = json!({"title":"Hello","publishDate":"2026-10-04","tags":[" Rust ","Rust"]});
        schemas.post.normalize(&mut post);
        schemas.post.validate(&post, "").unwrap();
        assert_eq!(post["tags"], json!(["Rust"]));
        post["tags"] = json!([" "]);
        schemas.post.normalize(&mut post);
        assert!(schemas.post.validate(&post, "").is_err());
    }
    #[test]
    fn historical_versions_use_the_same_parser() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        for version in ["v0.1.0", "v0.2.0"] {
            let root = fixtures.join(version).canonicalize().unwrap();
            let schemas = load(&root).unwrap();
            assert_eq!(
                schemas.config.field("copyright").is_some(),
                version == "v0.2.0"
            );
            assert!(schemas.post.field("exclude_in_search").is_none());
            assert!(schemas.post.field("publishDate").is_some());
        }
    }
    #[test]
    fn unknown_transform_is_rejected_without_execution() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let mut reader = Reader::new(&root).unwrap();
        let module = Module::parse(
            &root.join("test.ts"),
            "import {z} from 'astro/zod'; const x=z.string().transform(s=>fetch(s));".into(),
        )
        .unwrap();
        assert!(
            reader
                .zod_schema(&module, module.binding("x").unwrap(), 0)
                .is_err()
        );
    }
}
