use crate::application::flow_manager::{FlowManager, UpdateDraftRequest};
use crate::application::harbor::provider::HarborProvider;
use crate::application::harbor::types::{
    ConflictPolicy, ExportPolicy, HarborImportResourceResult, HarborResourceBundle, HarborScope,
};
use crate::domain::flow::models::FlowDraft;
use crate::error::{Error, Result};
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarborFlowPayload {
    pub name: String,
    pub description: Option<String>,
    pub flow_type: String,
    pub graph_json: Value,
    #[serde(default)]
    pub flow_id: Option<String>,
}

pub struct FlowHarborProvider {
    flow_manager: Arc<FlowManager>,
}

impl FlowHarborProvider {
    pub fn new(flow_manager: Arc<FlowManager>) -> Self {
        Self { flow_manager }
    }
}

#[async_trait]
impl HarborProvider for FlowHarborProvider {
    fn key(&self) -> &'static str {
        "flow"
    }

    fn validate(&self, resource: &HarborResourceBundle) -> Result<()> {
        if !resource.assets.is_empty() {
            return Err(Error::Validation(
                "Flow bundles must not include assets".to_string(),
            ));
        }

        let payload: HarborFlowPayload = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid flow bundle payload: {}", err)))?;

        validate_graph_json(&payload.graph_json)?;

        Ok(())
    }

    async fn export(
        &self,
        _realm_id: Uuid,
        scope: &HarborScope,
        _policy: ExportPolicy,
    ) -> Result<HarborResourceBundle> {
        let flow_id = match scope {
            HarborScope::Flow { flow_id } => *flow_id,
            _ => {
                return Err(Error::Validation(
                    "Flow export requires flow scope".to_string(),
                ))
            }
        };

        let draft = self.flow_manager.get_draft(flow_id).await?;
        let graph_json: Value = serde_json::from_str(&draft.graph_json)
            .map_err(|_| Error::Validation("Invalid flow graph JSON".to_string()))?;

        let payload = HarborFlowPayload {
            name: draft.name,
            description: draft.description,
            flow_type: draft.flow_type,
            graph_json,
            flow_id: Some(flow_id.to_string()),
        };

        let data = to_value(&payload)
            .map_err(|err| Error::System(format!("Failed to serialize flow: {}", err)))?;

        Ok(HarborResourceBundle {
            key: self.key().to_string(),
            data,
            assets: Vec::new(),
            meta: None,
        })
    }

    async fn import(
        &self,
        realm_id: Uuid,
        scope: &HarborScope,
        resource: &HarborResourceBundle,
        conflict_policy: ConflictPolicy,
        dry_run: bool,
        mut tx: Option<&mut dyn Transaction>,
    ) -> Result<HarborImportResourceResult> {
        let flow_id = match scope {
            HarborScope::Flow { flow_id } => *flow_id,
            _ => {
                return Err(Error::Validation(
                    "Flow import requires flow scope".to_string(),
                ))
            }
        };

        let payload: HarborFlowPayload = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid flow bundle payload: {}", err)))?;

        if let Some(payload_id) = payload.flow_id.as_deref() {
            if payload_id != flow_id.to_string() {
                return Err(Error::Validation(
                    "Flow bundle id does not match import scope".to_string(),
                ));
            }
        }

        let draft_exists = self.flow_manager.draft_exists(flow_id).await?;
        if draft_exists {
            match conflict_policy {
                ConflictPolicy::Skip => {
                    return Ok(HarborImportResourceResult {
                        key: self.key().to_string(),
                        status: "skipped".to_string(),
                        created: 0,
                        updated: 0,
                        errors: Vec::new(),
                        original_id: Some(flow_id.to_string()),
                        renamed_to: None,
                    });
                }
                ConflictPolicy::Rename => {
                    let new_flow_id = Uuid::new_v4();
                    let mut payload = payload;
                    let mut graph_json = payload.graph_json;
                    let mut map = std::collections::HashMap::new();
                    map.insert(flow_id.to_string(), new_flow_id.to_string());
                    rewrite_reference_ids(&mut graph_json, "flow_id", &map);
                    payload.graph_json = graph_json;
                    let mut result = create_flow_draft(
                        &self.flow_manager,
                        realm_id,
                        new_flow_id,
                        payload,
                        true,
                        dry_run,
                        tx.as_deref_mut(),
                    )
                    .await?;
                    result.original_id = Some(flow_id.to_string());
                    result.renamed_to = Some(new_flow_id.to_string());
                    return Ok(result);
                }
                ConflictPolicy::Overwrite => {}
            }
        }

        if draft_exists {
            let existing = self.flow_manager.get_draft(flow_id).await?;
            if existing.flow_type != payload.flow_type {
                return Err(Error::Validation(
                    "Flow type mismatch for existing flow".to_string(),
                ));
            }

            if dry_run {
                return Ok(HarborImportResourceResult {
                    key: self.key().to_string(),
                    status: "validated".to_string(),
                    created: 0,
                    updated: 1,
                    errors: Vec::new(),
                    original_id: Some(flow_id.to_string()),
                    renamed_to: None,
                });
            }

            self.flow_manager
                .update_draft_with_tx(
                    flow_id,
                    UpdateDraftRequest {
                        name: Some(payload.name),
                        description: payload.description,
                        graph_json: Some(payload.graph_json),
                    },
                    tx.as_deref_mut(),
                )
                .await?;

            return Ok(HarborImportResourceResult {
                key: self.key().to_string(),
                status: "updated".to_string(),
                created: 0,
                updated: 1,
                errors: Vec::new(),
                original_id: Some(flow_id.to_string()),
                renamed_to: None,
            });
        }

        create_flow_draft(
            &self.flow_manager,
            realm_id,
            flow_id,
            payload,
            false,
            dry_run,
            tx,
        )
        .await
    }
}

async fn create_flow_draft(
    flow_manager: &FlowManager,
    realm_id: Uuid,
    flow_id: Uuid,
    payload: HarborFlowPayload,
    renamed: bool,
    dry_run: bool,
    tx: Option<&mut dyn Transaction>,
) -> Result<HarborImportResourceResult> {
    if dry_run {
        return Ok(HarborImportResourceResult {
            key: "flow".to_string(),
            status: "validated".to_string(),
            created: 1,
            updated: 0,
            errors: Vec::new(),
            original_id: Some(flow_id.to_string()),
            renamed_to: None,
        });
    }

    let name = if renamed {
        format!("{} (imported)", payload.name)
    } else {
        payload.name
    };

    let draft = FlowDraft {
        id: flow_id,
        realm_id,
        name,
        description: payload.description,
        graph_json: payload.graph_json.to_string(),
        flow_type: payload.flow_type,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    flow_manager.create_draft_with_id_with_tx(draft, tx).await?;

    Ok(HarborImportResourceResult {
        key: "flow".to_string(),
        status: "created".to_string(),
        created: 1,
        updated: 0,
        errors: Vec::new(),
        original_id: Some(flow_id.to_string()),
        renamed_to: None,
    })
}

fn validate_graph_json(value: &Value) -> Result<()> {
    let Some(obj) = value.as_object() else {
        return Err(Error::Validation(
            "Flow graph must be a JSON object".to_string(),
        ));
    };

    match obj.get("nodes") {
        Some(Value::Array(_)) => {}
        _ => {
            return Err(Error::Validation(
                "Flow graph must contain nodes array".to_string(),
            ))
        }
    }

    match obj.get("edges") {
        Some(Value::Array(_)) => {}
        _ => {
            return Err(Error::Validation(
                "Flow graph must contain edges array".to_string(),
            ))
        }
    }

    Ok(())
}

fn rewrite_reference_ids(
    value: &mut Value,
    key: &str,
    map: &std::collections::HashMap<String, String>,
) {
    if let Some(obj) = value.as_object_mut() {
        if let Some(field) = obj.get_mut(key) {
            if let Some(value_str) = field.as_str() {
                if let Some(replacement) = map.get(value_str) {
                    *field = Value::String(replacement.clone());
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
