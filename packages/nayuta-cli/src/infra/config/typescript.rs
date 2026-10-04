use crate::infra::error::{Result, read, user};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct Module {
    pub path: PathBuf,
    pub source: String,
    pub ast: Value,
}

pub fn kind(node: &Value) -> &str {
    node["type"].as_str().unwrap_or_default()
}
pub fn name(node: &Value) -> &str {
    node["name"]
        .as_str()
        .or_else(|| node["value"].as_str())
        .unwrap_or_default()
}

pub fn unwrap(node: &Value) -> &Value {
    match kind(node) {
        "TSAsExpression"
        | "TSSatisfiesExpression"
        | "TSTypeAssertion"
        | "ParenthesizedExpression" => unwrap(&node["expression"]),
        _ => node,
    }
}

pub fn properties(node: &Value) -> impl Iterator<Item = (&str, &Value)> {
    node["properties"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| kind(p) == "Property" && !p["computed"].as_bool().unwrap_or(false))
        .map(|p| (name(&p["key"]), &p["value"]))
}

pub fn property<'a>(node: &'a Value, key: &str) -> Option<&'a Value> {
    properties(unwrap(node))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| value)
}

impl Module {
    pub fn parse(path: &Path, source: String) -> Result<Self> {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        if !parsed.errors.is_empty() {
            return Err(user(
                "unsupported",
                format!("{}: {:?}", path.display(), parsed.errors.first()),
            ));
        }
        let ast = serde_json::from_str(&parsed.program.to_estree_ts_json(false))?;
        Ok(Self {
            path: path.into(),
            source,
            ast,
        })
    }

    pub fn declarations(&self) -> impl Iterator<Item = &Value> {
        self.ast["body"].as_array().into_iter().flatten().map(|n| {
            if kind(n) == "ExportNamedDeclaration" && !n["declaration"].is_null() {
                &n["declaration"]
            } else {
                n
            }
        })
    }

    pub fn binding(&self, key: &str) -> Option<&Value> {
        self.declarations()
            .filter(|n| kind(n) == "VariableDeclaration")
            .flat_map(|n| n["declarations"].as_array().into_iter().flatten())
            .find(|n| name(&n["id"]) == key)
            .map(|n| &n["init"])
    }

    pub fn declaration(&self, key: &str) -> Option<&Value> {
        self.declarations().find(|n| name(&n["id"]) == key)
    }

    pub fn import(&self, local: &str) -> Option<(String, String)> {
        for node in self
            .declarations()
            .filter(|n| kind(n) == "ImportDeclaration")
        {
            for specifier in node["specifiers"].as_array().into_iter().flatten() {
                if name(&specifier["local"]) == local {
                    let imported = if kind(specifier) == "ImportDefaultSpecifier" {
                        "default"
                    } else {
                        name(&specifier["imported"])
                    };
                    return Some((node["source"]["value"].as_str()?.into(), imported.into()));
                }
            }
        }
        None
    }

    pub fn source_of(&self, node: &Value) -> &str {
        self.source
            .get(
                node["start"].as_u64().unwrap_or(0) as usize
                    ..node["end"].as_u64().unwrap_or(0) as usize,
            )
            .unwrap_or_default()
    }

    pub fn config_object(&self) -> Result<&Value> {
        let export = self
            .declarations()
            .find(|n| kind(n) == "ExportDefaultDeclaration")
            .ok_or_else(|| self.unsupported(&self.ast))?;
        let mut value = unwrap(&export["declaration"]);
        if kind(value) == "Identifier" {
            value = unwrap(
                self.binding(name(value))
                    .ok_or_else(|| self.unsupported(value))?,
            );
        }
        if kind(value) != "ObjectExpression" {
            return Err(self.unsupported(value));
        }
        Ok(value)
    }

    pub fn unsupported(&self, node: &Value) -> crate::infra::error::Error {
        user(
            "unsupported",
            format!("{}:{}: {}", self.path.display(), node["start"], kind(node)),
        )
    }
}

pub struct Reader {
    pub root: PathBuf,
    modules: HashMap<PathBuf, Module>,
    aliases: Value,
}

impl Reader {
    pub fn new(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let source = read(&root.join("tsconfig.json"))?;
        let module = Module::parse(
            &root.join("tsconfig.json"),
            format!("export default {source};"),
        )?;
        let mut reader = Self {
            root,
            modules: HashMap::new(),
            aliases: Value::Null,
        };
        reader.aliases = reader.value(&module, module.config_object()?, 0)?;
        Ok(reader)
    }

    pub fn module(&mut self, path: &Path) -> Result<Module> {
        let path = path.canonicalize()?;
        if !path.starts_with(&self.root) {
            return Err(user("unsupported", path.display().to_string()));
        }
        if let Some(module) = self.modules.get(&path) {
            return Ok(module.clone());
        }
        let module = Module::parse(&path, read(&path)?)?;
        self.modules.insert(path, module.clone());
        Ok(module)
    }

    pub fn resolve(&self, module: &Module, specifier: &str) -> Result<PathBuf> {
        let specifier = specifier.split('?').next().unwrap();
        let mut path = if specifier.starts_with('.') {
            module.path.parent().unwrap().join(specifier)
        } else {
            let aliases = self.aliases["compilerOptions"]["paths"]
                .as_object()
                .ok_or_else(|| user("unsupported", specifier))?;
            let (prefix, replacement) = aliases
                .iter()
                .filter_map(|(pattern, paths)| {
                    let prefix = pattern.trim_end_matches('*');
                    (if pattern.ends_with('*') {
                        specifier.starts_with(prefix)
                    } else {
                        specifier == pattern
                    })
                    .then(|| {
                        (
                            prefix,
                            paths[0].as_str().unwrap_or_default().trim_end_matches('*'),
                        )
                    })
                })
                .max_by_key(|(prefix, _)| prefix.len())
                .ok_or_else(|| user("unsupported", specifier))?;
            self.root.join(replacement).join(&specifier[prefix.len()..])
        };
        if path.is_file() {
            return Ok(path);
        }
        if path.with_extension("ts").is_file() {
            path.set_extension("ts");
            return Ok(path);
        }
        if path.join("index.ts").is_file() {
            return Ok(path.join("index.ts"));
        }
        Err(user("not_found", path.display().to_string()))
    }

    pub fn imported(&mut self, module: &Module, local: &str) -> Result<(Module, String)> {
        let (specifier, imported) = module
            .import(local)
            .ok_or_else(|| module.unsupported(&json!({"type":"Identifier","name":local})))?;
        let path = self.resolve(module, &specifier)?;
        Ok((self.module(&path)?, imported))
    }

    pub fn value(&mut self, module: &Module, expression: &Value, depth: usize) -> Result<Value> {
        if depth > 32 {
            return Err(module.unsupported(expression));
        }
        let node = unwrap(expression);
        match kind(node) {
            "Literal" if node.get("regex").is_none() => Ok(node["value"].clone()),
            "ArrayExpression" => Ok(Value::Array(
                node["elements"]
                    .as_array()
                    .ok_or_else(|| module.unsupported(node))?
                    .iter()
                    .map(|n| self.value(module, n, depth + 1))
                    .collect::<Result<_>>()?,
            )),
            "ObjectExpression" => {
                if node["properties"].as_array().is_some_and(|p| {
                    p.iter()
                        .any(|n| kind(n) != "Property" || n["computed"].as_bool() == Some(true))
                }) {
                    return Err(module.unsupported(node));
                }
                let mut object = serde_json::Map::new();
                for (key, value) in properties(node) {
                    if object.contains_key(key) {
                        return Err(module.unsupported(node));
                    }
                    object.insert(key.into(), self.value(module, value, depth + 1)?);
                }
                Ok(Value::Object(object))
            }
            "Identifier" => {
                let local = name(node);
                if let Some(value) = module.binding(local) {
                    return self.value(module, value, depth + 1);
                }
                if let Some((specifier, _)) = module.import(local)
                    && specifier.ends_with("?url")
                {
                    let path = self.resolve(module, &specifier)?.canonicalize()?;
                    if !path.starts_with(&self.root) {
                        return Err(module.unsupported(node));
                    }
                    return Ok(Value::String(format!(
                        "@asset:{}",
                        path.strip_prefix(&self.root)
                            .unwrap()
                            .to_string_lossy()
                            .replace('\\', "/")
                    )));
                }
                let (imported, key) = self.imported(module, local)?;
                let value = imported
                    .binding(&key)
                    .ok_or_else(|| imported.unsupported(node))?;
                self.value(&imported, value, depth + 1)
            }
            "NewExpression"
                if name(&node["callee"]) == "Date"
                    && node["arguments"].as_array().is_some_and(|a| a.len() == 1) =>
            {
                self.value(module, &node["arguments"][0], depth + 1)
            }
            "UnaryExpression" if node["operator"] == "-" => {
                let n = self
                    .value(module, &node["argument"], depth + 1)?
                    .as_f64()
                    .ok_or_else(|| module.unsupported(node))?;
                Ok(json!(-n))
            }
            "TemplateLiteral" if node["expressions"].as_array().is_some_and(|a| a.is_empty()) => {
                Ok(node["quasis"][0]["value"]["cooked"].clone())
            }
            _ => Err(module.unsupported(node)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jsonc_aliases_use_exact_and_longest_matches() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("tsconfig.json"),"{ // comment\n\"compilerOptions\": {\"paths\": {\"@type/*\": [\"src/*\"],\"@type/config\": [\"special.ts\"],},},}").unwrap();
        std::fs::write(temp.path().join("special.ts"), "export const x = 1;").unwrap();
        let reader = Reader::new(temp.path()).unwrap();
        let module =
            Module::parse(&temp.path().join("config.ts"), "export default {};".into()).unwrap();
        assert_eq!(
            reader.resolve(&module, "@type/config").unwrap(),
            temp.path().canonicalize().unwrap().join("special.ts")
        );
        assert!(reader.resolve(&module, "@type/config-extra").is_err());
    }
}
