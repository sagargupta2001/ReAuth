use crate::error::{Error, Result};
use jsonschema::JSONSchema;
use once_cell::sync::Lazy;
use serde_json::Value;

static BUNDLE_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../docs/schemas/harbor/bundle.schema.json"
    ))
    .expect("bundle schema");
    JSONSchema::compile(&schema).expect("compile bundle schema")
});

static THEME_RESOURCE_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../docs/schemas/harbor/resource-theme.schema.json"
    ))
    .expect("theme schema");
    JSONSchema::compile(&schema).expect("compile theme schema")
});

static CLIENT_RESOURCE_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../docs/schemas/harbor/resource-client.schema.json"
    ))
    .expect("client schema");
    JSONSchema::compile(&schema).expect("compile client schema")
});

static FLOW_RESOURCE_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../docs/schemas/harbor/resource-flow.schema.json"
    ))
    .expect("flow schema");
    JSONSchema::compile(&schema).expect("compile flow schema")
});

static ROLE_RESOURCE_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../docs/schemas/harbor/resource-role.schema.json"
    ))
    .expect("role schema");
    JSONSchema::compile(&schema).expect("compile role schema")
});

static REALM_RESOURCE_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../docs/schemas/harbor/resource-realm.schema.json"
    ))
    .expect("realm schema");
    JSONSchema::compile(&schema).expect("compile realm schema")
});

pub fn validate_bundle_schema(value: &Value) -> Result<()> {
    validate_with_schema(&BUNDLE_SCHEMA, value, "bundle")
}

pub fn validate_resource_schema(key: &str, value: &Value) -> Result<()> {
    match key {
        "theme" => validate_with_schema(&THEME_RESOURCE_SCHEMA, value, "theme resource"),
        "client" => validate_with_schema(&CLIENT_RESOURCE_SCHEMA, value, "client resource"),
        "flow" => validate_with_schema(&FLOW_RESOURCE_SCHEMA, value, "flow resource"),
        "role" => validate_with_schema(&ROLE_RESOURCE_SCHEMA, value, "role resource"),
        "realm" => validate_with_schema(&REALM_RESOURCE_SCHEMA, value, "realm resource"),
        _ => Ok(()),
    }
}

fn validate_with_schema(schema: &JSONSchema, value: &Value, label: &str) -> Result<()> {
    if let Err(errors) = schema.validate(value) {
        let mut messages = Vec::new();
        for error in errors.take(5) {
            messages.push(error.to_string());
        }
        return Err(Error::Validation(format!(
            "Invalid {} schema: {}",
            label,
            messages.join("; ")
        )));
    }
    Ok(())
}
