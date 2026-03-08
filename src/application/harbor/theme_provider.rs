use crate::application::harbor::provider::HarborProvider;
use crate::application::harbor::types::{
    ConflictPolicy, ExportPolicy, HarborAsset, HarborImportResourceResult, HarborResourceBundle,
    HarborScope, HarborThemeBindings, HarborThemeClientBinding, HarborThemeMeta,
    HarborThemeMetaTheme,
};
use crate::application::theme_service::ThemeResolverService;
use crate::domain::theme::ThemeDraft;
use crate::error::{Error, Result};
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde_json::{to_value, Value};
use std::sync::Arc;
use uuid::Uuid;

pub struct ThemeHarborProvider {
    theme_service: Arc<ThemeResolverService>,
}

impl ThemeHarborProvider {
    pub fn new(theme_service: Arc<ThemeResolverService>) -> Self {
        Self { theme_service }
    }
}

#[async_trait]
impl HarborProvider for ThemeHarborProvider {
    fn key(&self) -> &'static str {
        "theme"
    }

    fn validate(&self, resource: &HarborResourceBundle) -> Result<()> {
        let draft: ThemeDraft = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid theme bundle payload: {}", err)))?;

        if draft.nodes.is_empty() {
            return Err(Error::Validation("Theme nodes are required".to_string()));
        }

        for asset in &resource.assets {
            if asset.asset_type.is_none() {
                return Err(Error::Validation(
                    "Theme asset type is required".to_string(),
                ));
            }
            if asset.filename.trim().is_empty() {
                return Err(Error::Validation(
                    "Theme asset filename is required".to_string(),
                ));
            }
        }

        Ok(())
    }

    async fn export(
        &self,
        realm_id: Uuid,
        scope: &HarborScope,
        _policy: ExportPolicy,
    ) -> Result<HarborResourceBundle> {
        let theme_id = match scope {
            HarborScope::Theme { theme_id } => *theme_id,
            _ => {
                return Err(Error::Validation(
                    "Theme export requires theme scope".to_string(),
                ))
            }
        };

        let draft = self.theme_service.get_draft(realm_id, theme_id).await?;
        let theme = self
            .theme_service
            .get_theme(realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;
        let assets_meta = self.theme_service.list_assets(realm_id, theme_id).await?;
        let draft_exists = self.theme_service.draft_exists(realm_id, theme_id).await?;
        let default_binding = self.theme_service.resolve_binding(realm_id, None).await?;
        let default_binding_active = default_binding
            .as_ref()
            .map(|binding| binding.theme_id == theme_id)
            .unwrap_or(false);
        let mut client_bindings = self
            .theme_service
            .list_bindings_for_theme(realm_id, theme_id)
            .await?
            .into_iter()
            .filter_map(|binding| binding.client_id)
            .map(|client_id| HarborThemeClientBinding { client_id })
            .collect::<Vec<_>>();
        client_bindings.sort_by(|a, b| a.client_id.cmp(&b.client_id));

        let mut assets = Vec::new();
        for meta in assets_meta {
            let asset = self
                .theme_service
                .get_asset(&theme_id, &meta.id)
                .await?
                .ok_or_else(|| Error::NotFound("Theme asset not found".to_string()))?;
            assets.push(HarborAsset {
                id: Some(asset.id.to_string()),
                filename: asset.filename,
                mime_type: asset.mime_type,
                asset_type: Some(asset.asset_type),
                data_base64: STANDARD.encode(asset.data),
            });
        }

        let data = to_value(&draft)
            .map_err(|err| Error::System(format!("Failed to serialize theme draft: {}", err)))?;

        let meta = HarborThemeMeta {
            draft_exists,
            theme: Some(HarborThemeMetaTheme {
                name: theme.name,
                description: theme.description,
                is_system: theme.is_system,
            }),
            bindings: Some(HarborThemeBindings {
                default: default_binding_active,
                clients: client_bindings,
            }),
        };

        Ok(HarborResourceBundle {
            key: self.key().to_string(),
            data,
            assets,
            meta: Some(
                to_value(meta)
                    .map_err(|err| Error::System(format!("Failed to serialize meta: {}", err)))?,
            ),
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
        let theme_id = match scope {
            HarborScope::Theme { theme_id } => *theme_id,
            _ => {
                return Err(Error::Validation(
                    "Theme import requires theme scope".to_string(),
                ))
            }
        };

        let theme = if let Some(tx_ref) = tx.as_deref_mut() {
            self.theme_service
                .get_theme_with_tx(realm_id, &theme_id, Some(tx_ref))
                .await?
        } else {
            self.theme_service.get_theme(realm_id, &theme_id).await?
        };
        if theme.is_none() && !dry_run {
            return Err(Error::NotFound("Theme not found".to_string()));
        }

        if matches!(conflict_policy, ConflictPolicy::Rename) {
            return Err(Error::Validation(
                "Rename conflict policy is not supported for themes".to_string(),
            ));
        }

        let draft: ThemeDraft = serde_json::from_value(resource.data.clone())
            .map_err(|err| Error::Validation(format!("Invalid theme bundle payload: {}", err)))?;

        let incoming_draft_exists = resource
            .meta
            .as_ref()
            .and_then(|meta| serde_json::from_value::<HarborThemeMeta>(meta.clone()).ok())
            .map(|meta| meta.draft_exists)
            .unwrap_or(true);

        if matches!(conflict_policy, ConflictPolicy::Skip) && incoming_draft_exists {
            let exists = if let Some(tx_ref) = tx.as_deref_mut() {
                self.theme_service
                    .draft_exists_with_tx(realm_id, theme_id, Some(tx_ref))
                    .await?
            } else {
                self.theme_service.draft_exists(realm_id, theme_id).await?
            };
            if exists {
                return Ok(HarborImportResourceResult {
                    key: self.key().to_string(),
                    status: "skipped".to_string(),
                    created: 0,
                    updated: 0,
                    errors: Vec::new(),
                    original_id: None,
                    renamed_to: None,
                });
            }
        }

        let mut id_map = std::collections::HashMap::new();
        let mut created_assets = 0u32;
        for asset in &resource.assets {
            if asset.data_base64.trim().is_empty() {
                continue;
            }
            let data = STANDARD
                .decode(asset.data_base64.as_bytes())
                .map_err(|_| Error::Validation("Invalid asset data".to_string()))?;

            if dry_run {
                created_assets += 1;
                continue;
            }

            let asset_type = asset
                .asset_type
                .clone()
                .ok_or_else(|| Error::Validation("Theme asset type is required".to_string()))?;

            let created = self
                .theme_service
                .create_asset_with_tx(
                    realm_id,
                    theme_id,
                    asset_type,
                    asset.filename.clone(),
                    asset.mime_type.clone(),
                    data,
                    tx.as_deref_mut(),
                )
                .await?;
            if let Some(old_id) = asset.id.as_ref() {
                id_map.insert(old_id.clone(), created.id.to_string());
            }
            created_assets += 1;
        }

        if dry_run {
            return Ok(HarborImportResourceResult {
                key: self.key().to_string(),
                status: "validated".to_string(),
                created: created_assets,
                updated: 1,
                errors: Vec::new(),
                original_id: None,
                renamed_to: None,
            });
        }

        let mut draft = draft;
        for node in draft.nodes.iter_mut() {
            rewrite_blueprint_assets(&mut node.blueprint, &id_map);
        }

        self.theme_service
            .save_draft_with_tx(realm_id, theme_id, draft, tx)
            .await?;

        Ok(HarborImportResourceResult {
            key: self.key().to_string(),
            status: "imported".to_string(),
            created: created_assets,
            updated: 1,
            errors: Vec::new(),
            original_id: None,
            renamed_to: None,
        })
    }
}

fn rewrite_blueprint_assets(value: &mut Value, id_map: &std::collections::HashMap<String, String>) {
    if let Some(obj) = value.as_object_mut() {
        if let Some(props) = obj.get_mut("props").and_then(|value| value.as_object_mut()) {
            if let Some(asset_id) = props.get_mut("asset_id") {
                if let Some(asset_str) = asset_id.as_str() {
                    if let Some(replacement) = id_map.get(asset_str) {
                        *asset_id = Value::String(replacement.clone());
                    }
                }
            }
        }
        if let Some(children) = obj
            .get_mut("children")
            .and_then(|value| value.as_array_mut())
        {
            for child in children {
                rewrite_blueprint_assets(child, id_map);
            }
        }
        if let Some(slots) = obj.get_mut("slots").and_then(|value| value.as_object_mut()) {
            for slot in slots.values_mut() {
                rewrite_blueprint_assets(slot, id_map);
            }
        }
        if let Some(nodes) = obj.get_mut("nodes").and_then(|value| value.as_array_mut()) {
            for node in nodes {
                rewrite_blueprint_assets(node, id_map);
            }
        }
    } else if let Some(arr) = value.as_array_mut() {
        for entry in arr {
            rewrite_blueprint_assets(entry, id_map);
        }
    }
}
