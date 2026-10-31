use crate::infra::error::{Result, user};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub kind: Kind,
    pub optional: bool,
    pub default: Option<Value>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub integer: bool,
    pub trim: bool,
    pub unique: bool,
    pub pattern: Option<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Kind {
    String,
    Number,
    Boolean,
    Literal(Value),
    Array(Box<Schema>),
    Object(Vec<Field>),
    Union(Vec<Schema>),
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub schema: Schema,
}

impl Schema {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            optional: false,
            default: None,
            min: None,
            max: None,
            integer: false,
            trim: false,
            unique: false,
            pattern: None,
        }
    }

    pub fn fields(&self) -> &[Field] {
        match &self.kind {
            Kind::Object(fields) => fields,
            _ => &[],
        }
    }
    pub fn text_input(&self) -> bool {
        match &self.kind {
            Kind::String | Kind::Image | Kind::Literal(Value::String(_)) => true,
            Kind::Union(choices) => choices.iter().all(Self::text_input),
            _ => false,
        }
    }
    pub fn missing(&self, values: &Value, prefix: &str) -> Vec<String> {
        let mut missing = Vec::new();
        for field in self.fields() {
            let path = if prefix.is_empty() {
                field.name.clone()
            } else {
                format!("{prefix}.{}", field.name)
            };
            if let Some(value) = values.get(&field.name) {
                missing.extend(field.schema.missing(value, &path));
            } else if !field.schema.optional && field.schema.default.is_none() {
                missing.push(path);
            }
        }
        missing
    }

    pub fn field(&self, name: &str) -> Option<&Schema> {
        self.fields()
            .iter()
            .find(|f| f.name == name)
            .map(|f| &f.schema)
    }

    pub fn at(&self, path: &str) -> Option<&Schema> {
        path.split('.')
            .try_fold(self, |schema, segment| schema.field(segment))
    }
    pub fn at_mut(&mut self, path: &str) -> Option<&mut Schema> {
        let (head, tail) = path
            .split_once('.')
            .map_or((path, None), |(h, t)| (h, Some(t)));
        let Kind::Object(fields) = &mut self.kind else {
            return None;
        };
        let field = &mut fields.iter_mut().find(|f| f.name == head)?.schema;
        match tail {
            Some(tail) => field.at_mut(tail),
            None => Some(field),
        }
    }

    pub fn defaults(&self) -> Value {
        if let Some(value) = &self.default {
            return value.clone();
        }
        match &self.kind {
            Kind::Object(fields) => Value::Object(
                fields
                    .iter()
                    .filter_map(|f| {
                        let value = f.schema.defaults();
                        (!value.is_null()).then(|| (f.name.clone(), value))
                    })
                    .collect(),
            ),
            _ => Value::Null,
        }
    }

    pub fn normalize(&self, value: &mut Value) {
        match (&self.kind, &mut *value) {
            (Kind::String, Value::String(text)) if self.trim => *text = text.trim().into(),
            (Kind::Object(fields), Value::Object(map)) => {
                for field in fields {
                    if let Some(child) = map.get_mut(&field.name) {
                        field.schema.normalize(child);
                    }
                }
            }
            (Kind::Array(item), Value::Array(items)) => {
                for child in items.iter_mut() {
                    item.normalize(child);
                }
                if self.unique {
                    let mut unique = Vec::new();
                    for child in items.drain(..) {
                        if !unique.contains(&child) {
                            unique.push(child);
                        }
                    }
                    *items = unique;
                }
            }
            _ => {}
        }
    }

    pub fn validate(&self, value: &Value, path: &str) -> Result<()> {
        let fail = |reason: &str| user("validation", format!("{path}: {reason}"));
        let valid = match &self.kind {
            Kind::String | Kind::Image => value.is_string(),
            Kind::Number => value.is_number(),
            Kind::Boolean => value.is_boolean(),
            Kind::Literal(literal) => value == literal,
            Kind::Array(_) => value.is_array(),
            Kind::Object(_) => value.is_object(),
            Kind::Union(choices) => choices.iter().any(|s| s.validate(value, path).is_ok()),
        };
        if !valid {
            return Err(fail("type"));
        }
        if let Some((pattern, flags)) = &self.pattern
            && let Some(text) = value.as_str()
        {
            let regex =
                regress::Regex::with_flags(pattern, flags.as_str()).map_err(|_| fail("regex"))?;
            let utf16: Vec<_> = text.encode_utf16().collect();
            let matched = if flags.contains(['u', 'v']) {
                regex.find_from_utf16(&utf16, 0).next().is_some()
            } else {
                regex.find_from_ucs2(&utf16, 0).next().is_some()
            };
            if !matched {
                return Err(fail("pattern"));
            }
        }
        let magnitude = match value {
            Value::String(text) => Some(text.encode_utf16().count() as f64),
            Value::Array(items) => Some(items.len() as f64),
            Value::Number(number) => number.as_f64(),
            _ => None,
        };
        if let Some(number) = magnitude {
            if self.min.is_some_and(|min| number < min) {
                return Err(fail("minimum"));
            }
            if self.max.is_some_and(|max| number > max) {
                return Err(fail("maximum"));
            }
            if self.integer && number.fract() != 0.0 {
                return Err(fail("integer"));
            }
        }
        match &self.kind {
            Kind::Object(fields) => {
                for field in fields {
                    let child_path = if path.is_empty() {
                        field.name.clone()
                    } else {
                        format!("{path}.{}", field.name)
                    };
                    if let Some(child) = value.get(&field.name) {
                        field.schema.validate(child, &child_path)?;
                    } else if !field.schema.optional && field.schema.default.is_none() {
                        return Err(user("validation", format!("{child_path}: required")));
                    }
                }
            }
            Kind::Array(item) => {
                for (i, child) in value.as_array().unwrap().iter().enumerate() {
                    item.validate(child, &format!("{path}[{i}]"))?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn enum_values(&self) -> Option<Vec<Value>> {
        match &self.kind {
            Kind::Literal(value) => Some(vec![value.clone()]),
            Kind::Union(choices) => choices
                .iter()
                .map(|s| match &s.kind {
                    Kind::Literal(v) => Some(v.clone()),
                    _ => None,
                })
                .collect(),
            _ => None,
        }
    }
}

pub fn set(value: &mut Value, path: &str, next: Option<Value>) -> Result<()> {
    let mut segments = path.split('.').peekable();
    let mut current = value;
    while let Some(segment) = segments.next() {
        if segment.is_empty() {
            return Err(user("validation", path));
        }
        if !current.is_object() {
            *current = Value::Object(Default::default());
        }
        let map = current.as_object_mut().unwrap();
        if segments.peek().is_none() {
            if let Some(next) = next {
                map.insert(segment.into(), next);
            } else {
                map.remove(segment);
            }
            return Ok(());
        }
        current = map
            .entry(segment)
            .or_insert_with(|| Value::Object(Default::default()));
    }
    Err(user("validation", path))
}

pub fn get<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(value, |current, segment| current.get(segment))
}

pub fn apply_sets(schema: &Schema, value: &mut Value, sets: &[String]) -> Result<()> {
    for assignment in sets {
        let (path, raw) = assignment
            .split_once('=')
            .ok_or_else(|| user("validation", assignment))?;
        if schema.at(path).is_none() {
            return Err(user("validation", path));
        }
        set(
            value,
            path,
            Some(serde_json::from_str(raw).map_err(|_| user("invalid_json", path))?),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn javascript_regex_uses_utf16_and_unicode_flags() {
        let mut schema = Schema::new(Kind::String);
        schema.pattern = Some(("^.$".into(), String::new()));
        assert!(
            schema
                .validate(&Value::String("😀".into()), "emoji")
                .is_err()
        );
        schema.pattern = Some(("^.$".into(), "u".into()));
        schema
            .validate(&Value::String("😀".into()), "emoji")
            .unwrap();
    }
}
