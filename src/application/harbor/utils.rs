use crate::application::harbor::types::*;
use crate::error::{Error, Result};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub(crate) fn upgrade_bundle(mut bundle: HarborBundle) -> Result<HarborBundle> {
    if bundle.manifest.schema_version == super::service::HARBOR_SCHEMA_VERSION {
        return Ok(bundle);
    }

    if bundle.manifest.schema_version > super::service::HARBOR_SCHEMA_VERSION {
        return Err(Error::Validation(format!(
            "Unsupported schema version: {}",
            bundle.manifest.schema_version
        )));
    }

    match bundle.manifest.schema_version {
        0 => {
            bundle = upgrade_v0_to_v1(bundle);
            Ok(bundle)
        }
        _ => Err(Error::Validation(format!(
            "Unsupported schema version: {}",
            bundle.manifest.schema_version
        ))),
    }
}

pub(crate) fn upgrade_v0_to_v1(mut bundle: HarborBundle) -> HarborBundle {
    bundle.manifest.schema_version = super::service::HARBOR_SCHEMA_VERSION;
    bundle
}

pub(crate) fn rewrite_reference_ids(
    value: &mut serde_json::Value,
    key: &str,
    map: &std::collections::HashMap<String, String>,
) {
    if let Some(obj) = value.as_object_mut() {
        if let Some(field) = obj.get_mut(key) {
            if let Some(value_str) = field.as_str() {
                if let Some(replacement) = map.get(value_str) {
                    *field = serde_json::Value::String(replacement.clone());
                }
            }
        }
        for child in obj.values_mut() {
            rewrite_reference_ids(child, key, map);
        }
    } else if let Some(arr) = value.as_array_mut() {
        for entry in arr {
            rewrite_reference_ids(entry, key, map);
        }
    }
}

pub(crate) fn rewrite_realm_flow_bindings(
    value: &mut serde_json::Value,
    map: &std::collections::HashMap<String, String>,
) {
    let Some(bindings) = value
        .get_mut("flow_bindings")
        .and_then(|entry| entry.as_object_mut())
    else {
        return;
    };

    for key in [
        "browser_flow_id",
        "registration_flow_id",
        "direct_grant_flow_id",
        "reset_credentials_flow_id",
    ] {
        let Some(field) = bindings.get_mut(key) else {
            continue;
        };
        let Some(value_str) = field.as_str() else {
            continue;
        };
        if let Some(replacement) = map.get(value_str) {
            *field = serde_json::Value::String(replacement.clone());
        }
    }
}

pub(crate) fn rewrite_user_role_client_ids(
    value: &mut serde_json::Value,
    map: &std::collections::HashMap<String, String>,
) {
    let Some(direct_roles) = value
        .get_mut("direct_roles")
        .and_then(|entry| entry.as_array_mut())
    else {
        return;
    };

    for role in direct_roles {
        let Some(obj) = role.as_object_mut() else {
            continue;
        };
        let Some(field) = obj.get_mut("client_id") else {
            continue;
        };
        let Some(value_str) = field.as_str() else {
            continue;
        };
        if let Some(replacement) = map.get(value_str) {
            *field = serde_json::Value::String(replacement.clone());
        }
    }
}

pub(crate) fn rewrite_user_role_refs(
    value: &mut serde_json::Value,
    map: &std::collections::HashMap<String, String>,
) {
    let Some(direct_roles) = value
        .get_mut("direct_roles")
        .and_then(|entry| entry.as_array_mut())
    else {
        return;
    };

    for role in direct_roles {
        let Some(obj) = role.as_object_mut() else {
            continue;
        };
        let Some(name) = obj.get("name").and_then(|entry| entry.as_str()) else {
            continue;
        };
        let client_id = obj.get("client_id").and_then(|entry| entry.as_str());
        let encoded = encode_role_ref(name, client_id);
        let Some(replacement) = map.get(&encoded) else {
            continue;
        };
        let (replacement_name, replacement_client_id) = decode_role_ref(replacement);
        obj.insert(
            "name".to_string(),
            serde_json::Value::String(replacement_name.to_string()),
        );
        obj.insert(
            "client_id".to_string(),
            replacement_client_id
                .map(|value| serde_json::Value::String(value.to_string()))
                .unwrap_or(serde_json::Value::Null),
        );
    }
}

pub(crate) fn encode_role_ref(name: &str, client_id: Option<&str>) -> String {
    format!("{}::{}", client_id.unwrap_or(""), name)
}

pub(crate) fn decode_role_ref(value: &str) -> (&str, Option<&str>) {
    match value.split_once("::") {
        Some(("", name)) => (name, None),
        Some((client_id, name)) => (name, Some(client_id)),
        None => (value, None),
    }
}

pub(crate) fn parse_theme_meta(resource: &HarborResourceBundle) -> Result<HarborThemeMeta> {
    let meta_value = resource
        .meta
        .clone()
        .ok_or_else(|| Error::Validation("Theme bundle meta is required".to_string()))?;
    let meta: HarborThemeMeta = serde_json::from_value(meta_value)
        .map_err(|err| Error::Validation(format!("Invalid theme meta: {}", err)))?;

    if let Some(theme) = meta.theme.as_ref() {
        if theme.name.trim().is_empty() {
            return Err(Error::Validation("Theme meta name is required".to_string()));
        }
    }

    if let Some(bindings) = meta.bindings.as_ref() {
        for binding in &bindings.clients {
            if binding.client_id.trim().is_empty() {
                return Err(Error::Validation(
                    "Theme binding client_id is required".to_string(),
                ));
            }
        }
    }

    Ok(meta)
}

pub(crate) fn resolve_available_theme_name(
    existing: &HashMap<String, Uuid>,
    base: &str,
) -> Result<String> {
    for idx in 1..=1000 {
        let candidate = format!("{}-{}", base, idx);
        if !existing.contains_key(&candidate) {
            return Ok(candidate);
        }
    }
    Err(Error::Validation(
        "Unable to generate unique theme name".to_string(),
    ))
}

pub(crate) fn normalize_export_selection(selection: Option<Vec<String>>) -> Result<Vec<String>> {
    let mut normalized = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let raw = selection.unwrap_or_else(|| {
        vec![
            "client".to_string(),
            "flow".to_string(),
            "realm".to_string(),
            "role".to_string(),
            "theme".to_string(),
        ]
    });

    for entry in raw {
        let key = entry.trim().to_lowercase();
        let key = match key.as_str() {
            "client" | "clients" => "client",
            "flow" | "flows" => "flow",
            "realm" | "realms" | "settings" => "realm",
            "role" | "roles" | "rbac" => "role",
            "theme" | "themes" => "theme",
            "user" | "users" => "user",
            _ => {
                return Err(Error::Validation(format!(
                    "Unsupported export selection: {}",
                    entry
                )))
            }
        };
        if seen.insert(key.to_string()) {
            normalized.push(key.to_string());
        }
    }

    if normalized.is_empty() {
        return Err(Error::Validation(
            "Export selection must include at least one resource".to_string(),
        ));
    }

    normalized.sort();
    Ok(normalized)
}

pub(crate) fn normalize_bundle_for_export(bundle: &mut HarborBundle) {
    if let Some(selection) = bundle.manifest.selection.as_mut() {
        selection.sort();
    }

    for resource in &mut bundle.resources {
        if resource.key == "theme" {
            normalize_theme_meta(resource);
        }

        canonicalize_value(&mut resource.data);
        if let Some(meta) = resource.meta.as_mut() {
            canonicalize_value(meta);
        }

        resource.assets.sort_by_key(asset_sort_key);
    }

    bundle.resources.sort_by_key(resource_sort_key);
}

pub(crate) fn canonicalize_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let entries: Vec<(String, Value)> = std::mem::take(map).into_iter().collect();
            let mut entries = entries;
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut new_map = serde_json::Map::new();
            for (key, mut value) in entries {
                canonicalize_value(&mut value);
                new_map.insert(key, value);
            }
            *map = new_map;
        }
        Value::Array(list) => {
            for entry in list {
                canonicalize_value(entry);
            }
        }
        _ => {}
    }
}

pub(crate) fn resource_sort_key(resource: &HarborResourceBundle) -> (String, String) {
    let secondary = match resource.key.as_str() {
        "client" => get_string_field(&resource.data, "client_id"),
        "flow" => get_string_field(&resource.data, "flow_id")
            .or_else(|| get_string_field(&resource.data, "name")),
        "theme" => resource
            .meta
            .as_ref()
            .and_then(|meta| serde_json::from_value::<HarborThemeMeta>(meta.clone()).ok())
            .and_then(|meta| meta.theme.map(|theme| theme.name)),
        _ => get_string_field(&resource.data, "id"),
    }
    .unwrap_or_default();

    (resource.key.clone(), secondary)
}

pub(crate) fn asset_sort_key(asset: &HarborAsset) -> (String, String, String) {
    (
        asset.id.clone().unwrap_or_default(),
        asset.filename.clone(),
        asset.mime_type.clone(),
    )
}

pub(crate) fn get_string_field(value: &Value, field: &str) -> Option<String> {
    value
        .as_object()
        .and_then(|obj| obj.get(field))
        .and_then(|val| val.as_str())
        .map(|val| val.to_string())
}

pub(crate) fn normalize_theme_meta(resource: &mut HarborResourceBundle) {
    let Some(meta_value) = resource.meta.as_ref() else {
        return;
    };

    let mut meta: HarborThemeMeta = match serde_json::from_value(meta_value.clone()) {
        Ok(meta) => meta,
        Err(_) => return,
    };

    if let Some(bindings) = meta.bindings.as_mut() {
        bindings
            .clients
            .sort_by(|a, b| a.client_id.cmp(&b.client_id));
    }

    if let Ok(updated) = serde_json::to_value(meta) {
        resource.meta = Some(updated);
    }
}

pub(crate) fn summarize_import_counts(result: &HarborImportResult) -> (i64, i64) {
    let mut created = 0i64;
    let mut updated = 0i64;
    for resource in &result.resources {
        created += resource.created as i64;
        updated += resource.updated as i64;
    }
    (created, updated)
}

pub(crate) fn scope_label(scope: &HarborScope) -> &'static str {
    match scope {
        HarborScope::Theme { .. } => "theme",
        HarborScope::Client { .. } => "client",
        HarborScope::Flow { .. } => "flow",
        HarborScope::User { .. } => "user",
        HarborScope::Role { .. } => "role",
        HarborScope::FullRealm => "full_realm",
    }
}

pub(crate) fn conflict_policy_label(policy: ConflictPolicy) -> &'static str {
    match policy {
        ConflictPolicy::Skip => "skip",
        ConflictPolicy::Overwrite => "overwrite",
        ConflictPolicy::Rename => "rename",
    }
}
