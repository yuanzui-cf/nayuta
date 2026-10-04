use crate::{
    core::schema::{Kind, Schema, get, set},
    infra::error::{Result, user},
};
use serde_json::{Value, json};

#[derive(Clone)]
pub struct Field {
    pub path: String,
    pub schema: Schema,
    pub readonly: bool,
}

#[derive(Clone)]
pub struct Form {
    pub schema: Schema,
    pub values: Value,
    pub original: Value,
    pub readonly: Vec<String>,
    pub inherited: Value,
    pub focus: usize,
    pub scroll: usize,
}

impl Form {
    pub fn new(schema: Schema, values: Value) -> Self {
        Self {
            schema,
            original: values.clone(),
            values,
            readonly: Vec::new(),
            inherited: json!({}),
            focus: 0,
            scroll: 0,
        }
    }
    pub fn dirty(&self) -> bool {
        self.original != self.values
    }
    pub fn fields(&self) -> Vec<Field> {
        let mut fields = Vec::new();
        flatten(&self.schema, "", &mut fields);
        let enabled = get(&self.values, "copyright.enabled")
            .or_else(|| get(&self.inherited, "copyright.enabled"))
            == Some(&Value::Bool(true));
        fields.retain(|f| f.path != "copyright.license" || enabled);
        for field in &mut fields {
            field.readonly = self
                .readonly
                .iter()
                .any(|r| field.path == *r || field.path.starts_with(&format!("{r}.")));
        }
        fields
    }
    pub fn apply(&mut self, field: &Field, raw: &str) -> Result<()> {
        if field.readonly {
            return Err(user("read_only", &field.path));
        }
        let value = match &field.schema.kind {
            Kind::String | Kind::Image => Value::String(raw.into()),
            Kind::Union(choices)
                if choices.iter().any(|s| matches!(s.kind, Kind::String))
                    && field.schema.enum_values().is_none() =>
            {
                Value::String(raw.into())
            }
            _ => serde_json::from_str(raw).map_err(|_| user("invalid_json", &field.path))?,
        };
        let mut value = value;
        field.schema.normalize(&mut value);
        field.schema.validate(&value, &field.path)?;
        set(&mut self.values, &field.path, Some(value))
    }
    pub fn cycle(&mut self, field: &Field, reverse: bool) -> Result<bool> {
        if field.readonly {
            return Err(user("read_only", &field.path));
        }
        let values = if matches!(field.schema.kind, Kind::Boolean) {
            Some(vec![json!(true), json!(false)])
        } else {
            field.schema.enum_values()
        };
        let Some(values) = values else {
            return Ok(false);
        };
        let mut choices: Vec<_> = values.into_iter().map(Some).collect();
        if field.schema.optional || field.schema.default.is_some() {
            choices.insert(0, None);
        }
        let current = get(&self.values, &field.path);
        let index = choices.iter().position(|v| v.as_ref() == current);
        let index = match index {
            Some(i) if reverse => (i + choices.len() - 1) % choices.len(),
            Some(i) => (i + 1) % choices.len(),
            None => 0,
        };
        set(&mut self.values, &field.path, choices[index].clone())?;
        Ok(true)
    }
    pub fn unset(&mut self, field: &Field) -> Result<()> {
        if field.readonly {
            return Err(user("read_only", &field.path));
        }
        if !field.schema.optional && field.schema.default.is_none() {
            return Err(user("required", &field.path));
        }
        set(&mut self.values, &field.path, None)
    }
}

fn flatten(schema: &Schema, prefix: &str, out: &mut Vec<Field>) {
    if let Kind::Object(fields) = &schema.kind {
        for field in fields {
            flatten(
                &field.schema,
                &if prefix.is_empty() {
                    field.name.clone()
                } else {
                    format!("{prefix}.{}", field.name)
                },
                out,
            );
        }
    } else {
        out.push(Field {
            path: prefix.into(),
            schema: schema.clone(),
            readonly: false,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::schema::Field as SchemaField;
    #[test]
    fn copyright_visibility_preserves_hidden_values_and_inheritance() {
        let mut enabled = Schema::new(Kind::Boolean);
        enabled.optional = true;
        let mut license = Schema::new(Kind::String);
        license.optional = true;
        let schema = Schema::new(Kind::Object(vec![SchemaField {
            name: "copyright".into(),
            schema: Schema::new(Kind::Object(vec![
                SchemaField {
                    name: "enabled".into(),
                    schema: enabled,
                },
                SchemaField {
                    name: "license".into(),
                    schema: license,
                },
            ])),
        }]));
        let mut form = Form::new(schema, json!({"copyright":{"license":"CC BY 4.0"}}));
        assert_eq!(form.fields().len(), 1);
        form.inherited = json!({"copyright":{"enabled":true}});
        assert_eq!(form.fields().len(), 2);
        form.cycle(&form.fields()[0], false).unwrap();
        form.cycle(&form.fields()[0], false).unwrap();
        assert_eq!(form.fields().len(), 1);
        assert_eq!(form.values["copyright"]["license"], "CC BY 4.0");
    }
}
