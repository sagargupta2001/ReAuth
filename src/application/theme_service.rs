use crate::constants::DEFAULT_THEME_NAME;
use crate::domain::theme::{
    Theme, ThemeAsset, ThemeAssetRef, ThemeBinding, ThemeBlock, ThemeDraft, ThemeDraftNode,
    ThemeLayout, ThemeNode, ThemeSnapshot, ThemeTokens, ThemeVersion,
};
use crate::domain::theme_pages::{self, ThemePageTemplate};
use crate::error::{Error, Result};
use crate::ports::theme_repository::ThemeRepository;

struct ThemeBundle<'a> {
    theme: &'a Theme,
    tokens: &'a ThemeTokens,
    layout: &'a ThemeLayout,
    nodes: &'a [ThemeNode],
    version: &'a ThemeVersion,
    binding: Option<&'a ThemeBinding>,
}
use crate::ports::transaction_manager::Transaction;
use crate::ports::transaction_manager::TransactionManager;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::warn;
use uuid::Uuid;

pub struct ThemeResolverService {
    repo: Arc<dyn ThemeRepository>,
    tx_manager: Arc<dyn TransactionManager>,
}

impl ThemeResolverService {
    pub fn new(repo: Arc<dyn ThemeRepository>, tx_manager: Arc<dyn TransactionManager>) -> Self {
        Self { repo, tx_manager }
    }

    pub async fn resolve_snapshot(
        &self,
        realm_id: Uuid,
        realm_ref: &str,
        client_id: Option<&str>,
        node_key: Option<&str>,
    ) -> Result<ThemeSnapshot> {
        let binding = self.resolve_binding(realm_id, client_id).await?;

        let Some(binding) = binding else {
            return Ok(self.default_snapshot());
        };

        let version = self
            .repo
            .get_version(&binding.theme_id, &binding.active_version_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme version not found".to_string()))?;

        let requested_key = node_key.unwrap_or("login");
        let draft_payload = parse_version_payload(&version.snapshot_json);

        let (tokens, layout, blocks) = if let Some(payload) = draft_payload {
            let (blocks, _) = payload
                .nodes
                .iter()
                .find(|node| node.node_key == requested_key)
                .and_then(|node| parse_blueprint_value(node.blueprint.clone()))
                .unwrap_or_else(|| default_page_blocks(requested_key));
            (payload.tokens, payload.layout, blocks)
        } else {
            let tokens = self
                .repo
                .get_tokens(&binding.theme_id)
                .await?
                .and_then(|t| parse_json(&t.tokens_json, "theme_tokens"))
                .unwrap_or_else(default_tokens);
            let (blocks, layout_hint) =
                if let Some(node) = self.repo.get_node(&binding.theme_id, requested_key).await? {
                    parse_blueprint(&node.blueprint_json).unwrap_or_else(|| (Vec::new(), None))
                } else {
                    default_page_blocks(requested_key)
                };
            let layout = resolve_layout(&self.repo, binding.theme_id, layout_hint).await?;
            (tokens, layout, blocks)
        };

        let assets = self
            .repo
            .list_assets(&binding.theme_id)
            .await?
            .into_iter()
            .map(|asset| ThemeAssetRef {
                id: asset.id,
                filename: asset.filename,
                mime_type: asset.mime_type,
                url: format!(
                    "/api/realms/{}/theme/{}/assets/{}",
                    realm_ref, binding.theme_id, asset.id
                ),
                checksum: asset.checksum,
                byte_size: asset.byte_size,
            })
            .collect();

        Ok(ThemeSnapshot {
            theme_id: binding.theme_id,
            version_id: version.id,
            tokens,
            layout,
            blocks,
            assets,
        })
    }

    pub async fn create_theme(
        &self,
        realm_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<Theme> {
        self.create_theme_internal(realm_id, name, description, false)
            .await
    }

    pub async fn create_system_theme_in_tx(
        &self,
        realm_id: Uuid,
        tx: &mut dyn Transaction,
    ) -> Result<Theme> {
        self.create_theme_in_tx(
            realm_id,
            DEFAULT_THEME_NAME.to_string(),
            Some("System default theme".to_string()),
            true,
            tx,
        )
        .await
    }

    pub async fn ensure_default_theme(&self, realm_id: Uuid) -> Result<Theme> {
        let mut existing = self
            .repo
            .list_themes(&realm_id)
            .await?
            .into_iter()
            .find(|theme| theme.name == DEFAULT_THEME_NAME);
        if let Some(theme) = existing.as_mut() {
            if !theme.is_system {
                self.repo.set_theme_system(&theme.id, true, None).await?;
                theme.is_system = true;
            }
            self.ensure_theme_pages(theme.id).await?;
            return Ok(theme.clone());
        }

        self.create_theme_internal(
            realm_id,
            DEFAULT_THEME_NAME.to_string(),
            Some("System default theme".to_string()),
            true,
        )
        .await
    }

    async fn create_theme_internal(
        &self,
        realm_id: Uuid,
        name: String,
        description: Option<String>,
        is_system: bool,
    ) -> Result<Theme> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(Error::Validation("Theme name is required".to_string()));
        }

        let description = description.and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

        let theme = Theme {
            id: Uuid::new_v4(),
            realm_id,
            name,
            description,
            is_system,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        let tokens_value = default_tokens();
        let layout_value = default_layout();
        let page_templates = theme_pages::system_pages();

        let tokens = ThemeTokens {
            id: Uuid::new_v4(),
            theme_id: theme.id,
            tokens_json: serde_json::to_string(&tokens_value)
                .map_err(|err| Error::Unexpected(err.into()))?,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        let layout = ThemeLayout {
            id: Uuid::new_v4(),
            theme_id: theme.id,
            name: "default".to_string(),
            layout_json: serde_json::to_string(&layout_value)
                .map_err(|err| Error::Unexpected(err.into()))?,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };
        let nodes = if is_system {
            build_theme_nodes(theme.id, &page_templates)?
        } else {
            Vec::new()
        };

        let binding_exists = self.repo.get_binding(&realm_id, None).await?.is_some();
        let status = if binding_exists {
            "published"
        } else {
            "active"
        };

        let version = ThemeVersion {
            id: Uuid::new_v4(),
            theme_id: theme.id,
            version_number: 1,
            status: status.to_string(),
            snapshot_json: serde_json::to_string(&json!({
                "tokens": tokens_value,
                "layout": layout_value,
                "blocks": [],
            }))
            .map_err(|err| Error::Unexpected(err.into()))?,
            created_at: "".to_string(),
        };

        let binding = if binding_exists {
            None
        } else {
            Some(ThemeBinding {
                id: Uuid::new_v4(),
                realm_id,
                client_id: None,
                theme_id: theme.id,
                active_version_id: version.id,
                created_at: "".to_string(),
                updated_at: "".to_string(),
            })
        };

        let mut tx = self.tx_manager.begin().await?;
        let bundle = ThemeBundle {
            theme: &theme,
            tokens: &tokens,
            layout: &layout,
            nodes: &nodes,
            version: &version,
            binding: binding.as_ref(),
        };

        let result = self.persist_theme_bundle(bundle, &mut *tx).await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                if theme.is_system {
                    self.ensure_theme_pages(theme.id).await?;
                }
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        self.repo
            .find_theme(&realm_id, &theme.id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found after creation".to_string()))
    }

    pub async fn update_theme(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Theme> {
        if name.is_none() && description.is_none() {
            return Err(Error::Validation(
                "At least one field must be updated".to_string(),
            ));
        }

        let mut theme = self
            .repo
            .find_theme(&realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

        if let Some(name) = name {
            let trimmed = name.trim().to_string();
            if trimmed.is_empty() {
                return Err(Error::Validation("Theme name is required".to_string()));
            }
            theme.name = trimmed;
        }

        if let Some(description) = description {
            let trimmed = description.trim().to_string();
            theme.description = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
        }

        self.repo.update_theme(&theme, None).await?;
        self.repo
            .find_theme(&realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found after update".to_string()))
    }

    pub async fn activate_version(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
        version_id: Uuid,
    ) -> Result<()> {
        let theme = self
            .repo
            .find_theme(&realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

        let version = self
            .repo
            .get_version(&theme.id, &version_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme version not found".to_string()))?;

        let existing_binding = self.repo.get_binding(&realm_id, None).await?;
        let binding_id = existing_binding
            .as_ref()
            .map(|binding| binding.id)
            .unwrap_or_else(Uuid::new_v4);
        let previous_active = existing_binding.map(|binding| binding.active_version_id);

        let binding = ThemeBinding {
            id: binding_id,
            realm_id,
            client_id: None,
            theme_id: theme.id,
            active_version_id: version.id,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            if let Some(previous) = previous_active {
                if previous != version.id {
                    self.repo
                        .set_version_status(&previous, "published", Some(&mut *tx))
                        .await?;
                }
            }

            self.repo
                .set_version_status(&version.id, "active", Some(&mut *tx))
                .await?;
            self.repo.upsert_binding(&binding, Some(&mut *tx)).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => self.tx_manager.commit(tx).await?,
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        Ok(())
    }

    pub async fn publish_theme(&self, realm_id: Uuid, theme_id: Uuid) -> Result<ThemeVersion> {
        let theme = self
            .repo
            .find_theme(&realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

        let existing_versions = self.repo.list_versions(&theme.id).await?;
        let next_version = existing_versions
            .iter()
            .map(|version| version.version_number)
            .max()
            .unwrap_or(0)
            + 1;

        let draft = self.get_draft(realm_id, theme.id).await?;

        let binding = self.repo.get_binding(&realm_id, None).await?;
        let should_activate = match binding.as_ref() {
            Some(existing) => existing.theme_id == theme.id,
            None => true,
        };
        let status = if should_activate {
            "active"
        } else {
            "published"
        };

        let version = ThemeVersion {
            id: Uuid::new_v4(),
            theme_id: theme.id,
            version_number: next_version,
            status: status.to_string(),
            snapshot_json: serde_json::to_string(&draft)
                .map_err(|err| Error::Unexpected(err.into()))?,
            created_at: "".to_string(),
        };

        let previous_active = binding
            .as_ref()
            .filter(|existing| existing.theme_id == theme.id)
            .map(|existing| existing.active_version_id);

        let binding = if should_activate {
            let binding_id = binding
                .as_ref()
                .map(|existing| existing.id)
                .unwrap_or_else(Uuid::new_v4);
            Some(ThemeBinding {
                id: binding_id,
                realm_id,
                client_id: None,
                theme_id: theme.id,
                active_version_id: version.id,
                created_at: "".to_string(),
                updated_at: "".to_string(),
            })
        } else {
            None
        };

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.repo.create_version(&version, Some(&mut *tx)).await?;
            if should_activate {
                if let Some(previous) = previous_active {
                    if previous != version.id {
                        self.repo
                            .set_version_status(&previous, "published", Some(&mut *tx))
                            .await?;
                    }
                }
                self.repo
                    .set_version_status(&version.id, "active", Some(&mut *tx))
                    .await?;
                if let Some(binding) = binding.as_ref() {
                    self.repo.upsert_binding(binding, Some(&mut *tx)).await?;
                }
            }
            Ok(())
        }
        .await;

        match result {
            Ok(()) => self.tx_manager.commit(tx).await?,
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        self.repo
            .get_version(&theme.id, &version.id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme version not found".to_string()))
    }

    pub async fn get_draft(&self, realm_id: Uuid, theme_id: Uuid) -> Result<ThemeDraft> {
        let theme = self
            .repo
            .find_theme(&realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

        let tokens = self
            .repo
            .get_tokens(&theme.id)
            .await?
            .map(|tokens| parse_required_json(&tokens.tokens_json, "theme_tokens"))
            .transpose()?
            .unwrap_or_else(default_tokens);

        let layout = self
            .repo
            .get_layout(&theme.id, "default")
            .await?
            .map(|layout| parse_required_json(&layout.layout_json, "theme_layout"))
            .transpose()?
            .unwrap_or_else(default_layout);

        let nodes = self
            .repo
            .list_nodes(&theme.id)
            .await?
            .into_iter()
            .map(|node| {
                let blueprint = parse_required_json(&node.blueprint_json, "theme_node")
                    .unwrap_or_else(|_| json!([]));
                ThemeDraftNode {
                    node_key: node.node_key,
                    blueprint,
                }
            })
            .collect();

        Ok(ThemeDraft {
            tokens,
            layout,
            nodes,
        })
    }

    pub async fn start_draft_from_version(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
        version_id: Uuid,
    ) -> Result<ThemeDraft> {
        let theme = self
            .repo
            .find_theme(&realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

        let version = self
            .repo
            .get_version(&theme.id, &version_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme version not found".to_string()))?;

        let draft = parse_version_payload(&version.snapshot_json).ok_or_else(|| {
            Error::Validation("Theme version payload could not be restored".to_string())
        })?;

        self.save_draft(realm_id, theme_id, draft.clone()).await?;

        Ok(draft)
    }

    pub async fn save_draft(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
        draft: ThemeDraft,
    ) -> Result<()> {
        let theme = self
            .repo
            .find_theme(&realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

        let tokens_json = serde_json::to_string(&draft.tokens)
            .map_err(|err| Error::Validation(format!("Invalid theme tokens: {}", err)))?;
        let layout_json = serde_json::to_string(&draft.layout)
            .map_err(|err| Error::Validation(format!("Invalid theme layout: {}", err)))?;

        let tokens = ThemeTokens {
            id: Uuid::new_v4(),
            theme_id: theme.id,
            tokens_json,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        let layout = ThemeLayout {
            id: Uuid::new_v4(),
            theme_id: theme.id,
            name: "default".to_string(),
            layout_json,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        let nodes: Vec<ThemeNode> = draft
            .nodes
            .into_iter()
            .map(|node| ThemeNode {
                id: Uuid::new_v4(),
                theme_id: theme.id,
                node_key: node.node_key,
                blueprint_json: serde_json::to_string(&node.blueprint)
                    .unwrap_or_else(|_| "[]".to_string()),
                created_at: "".to_string(),
                updated_at: "".to_string(),
            })
            .collect();

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.repo.upsert_tokens(&tokens, Some(&mut *tx)).await?;
            self.repo.upsert_layout(&layout, Some(&mut *tx)).await?;
            let existing_nodes = self.repo.list_nodes(&theme.id).await?;
            let mut draft_keys = std::collections::HashSet::new();
            for node in &nodes {
                let key = node.node_key.trim();
                if key.is_empty() {
                    return Err(Error::Validation("Theme node key is required".to_string()));
                }
                if !theme_pages::is_valid_page(key) {
                    return Err(Error::Validation(format!(
                        "Unknown theme page key: {}",
                        key
                    )));
                }
                draft_keys.insert(key.to_string());
                self.repo.upsert_node(node, Some(&mut *tx)).await?;
            }
            if !theme.is_system {
                for existing in existing_nodes {
                    if !draft_keys.contains(&existing.node_key) {
                        self.repo
                            .delete_node(&theme.id, &existing.node_key, Some(&mut *tx))
                            .await?;
                    }
                }
            }
            Ok(())
        }
        .await;

        match result {
            Ok(()) => self.tx_manager.commit(tx).await?,
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        Ok(())
    }

    pub async fn list_assets(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
    ) -> Result<Vec<crate::domain::theme::ThemeAssetMeta>> {
        let theme = self
            .repo
            .find_theme(&realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

        self.repo.list_assets(&theme.id).await
    }

    pub async fn create_asset(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
        asset_type: String,
        filename: String,
        mime_type: String,
        data: Vec<u8>,
    ) -> Result<crate::domain::theme::ThemeAssetMeta> {
        let theme = self
            .repo
            .find_theme(&realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

        let checksum = if data.is_empty() {
            None
        } else {
            let digest = Sha256::digest(&data);
            Some(hex::encode(digest))
        };

        let asset = crate::domain::theme::ThemeAsset {
            id: Uuid::new_v4(),
            theme_id: theme.id,
            asset_type,
            filename,
            mime_type,
            byte_size: data.len() as i64,
            checksum,
            data,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        self.repo.create_asset(&asset, None).await?;

        Ok(crate::domain::theme::ThemeAssetMeta {
            id: asset.id,
            theme_id: asset.theme_id,
            asset_type: asset.asset_type,
            filename: asset.filename,
            mime_type: asset.mime_type,
            byte_size: asset.byte_size,
            checksum: asset.checksum,
            created_at: asset.created_at,
            updated_at: asset.updated_at,
        })
    }

    pub async fn preview_snapshot(
        &self,
        realm_id: Uuid,
        realm_ref: &str,
        theme_id: Uuid,
        node_key: Option<&str>,
    ) -> Result<ThemeSnapshot> {
        let theme = self
            .repo
            .find_theme(&realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

        self.build_snapshot_for_theme(&theme.id, realm_ref, node_key)
            .await
    }

    async fn build_snapshot_for_theme(
        &self,
        theme_id: &Uuid,
        realm_ref: &str,
        node_key: Option<&str>,
    ) -> Result<ThemeSnapshot> {
        let tokens = self
            .repo
            .get_tokens(theme_id)
            .await?
            .map(|tokens| parse_required_json(&tokens.tokens_json, "theme_tokens"))
            .transpose()?
            .unwrap_or_else(default_tokens);

        let nodes = self.repo.list_nodes(theme_id).await?;
        let requested_key = node_key.unwrap_or("login");
        let (blocks, layout_hint) = nodes
            .iter()
            .find(|node| node.node_key == requested_key)
            .and_then(|node| parse_blueprint(&node.blueprint_json))
            .unwrap_or_else(|| default_page_blocks(requested_key));

        let layout = if let Some(layout_name) = layout_hint {
            if let Some(layout) = self.repo.get_layout(theme_id, &layout_name).await? {
                parse_required_json(&layout.layout_json, "theme_layout")?
            } else {
                default_layout()
            }
        } else if let Some(layout) = self.repo.get_layout(theme_id, "default").await? {
            parse_required_json(&layout.layout_json, "theme_layout")?
        } else {
            default_layout()
        };

        let assets = self
            .repo
            .list_assets(theme_id)
            .await?
            .into_iter()
            .map(|asset| ThemeAssetRef {
                id: asset.id,
                filename: asset.filename,
                mime_type: asset.mime_type,
                url: format!(
                    "/api/realms/{}/theme/{}/assets/{}",
                    realm_ref, theme_id, asset.id
                ),
                checksum: asset.checksum,
                byte_size: asset.byte_size,
            })
            .collect();

        Ok(ThemeSnapshot {
            theme_id: *theme_id,
            version_id: Uuid::nil(),
            tokens,
            layout,
            blocks,
            assets,
        })
    }

    fn default_snapshot(&self) -> ThemeSnapshot {
        let (blocks, _) = default_page_blocks("login");
        ThemeSnapshot {
            theme_id: Uuid::nil(),
            version_id: Uuid::nil(),
            tokens: default_tokens(),
            layout: default_layout(),
            blocks,
            assets: Vec::new(),
        }
    }

    pub fn list_pages(&self) -> Vec<ThemePageTemplate> {
        theme_pages::system_pages()
    }

    pub async fn list_pages_for_theme(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
    ) -> Result<Vec<ThemePageTemplate>> {
        let theme = self
            .repo
            .find_theme(&realm_id, &theme_id)
            .await?
            .ok_or_else(|| Error::NotFound("Theme not found".to_string()))?;

        let mut pages = theme_pages::system_pages();
        let mut known_keys: std::collections::HashSet<String> =
            pages.iter().map(|page| page.key.clone()).collect();

        let nodes = self.repo.list_nodes(&theme.id).await?;
        for node in nodes {
            if !theme_pages::is_custom_page(&node.node_key) {
                continue;
            }
            if known_keys.contains(&node.node_key) {
                continue;
            }
            let blueprint = parse_json(&node.blueprint_json, "theme_blueprint")
                .unwrap_or_else(theme_pages::default_page_blueprint_fallback);
            pages.push(theme_pages::custom_page_template(&node.node_key, blueprint));
            known_keys.insert(node.node_key);
        }

        Ok(pages)
    }

    async fn create_theme_in_tx(
        &self,
        realm_id: Uuid,
        name: String,
        description: Option<String>,
        is_system: bool,
        tx: &mut dyn Transaction,
    ) -> Result<Theme> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(Error::Validation("Theme name is required".to_string()));
        }

        let description = description.and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

        let theme = Theme {
            id: Uuid::new_v4(),
            realm_id,
            name,
            description,
            is_system,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        let tokens_value = default_tokens();
        let layout_value = default_layout();
        let page_templates = theme_pages::system_pages();

        let tokens = ThemeTokens {
            id: Uuid::new_v4(),
            theme_id: theme.id,
            tokens_json: serde_json::to_string(&tokens_value)
                .map_err(|err| Error::Unexpected(err.into()))?,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        let layout = ThemeLayout {
            id: Uuid::new_v4(),
            theme_id: theme.id,
            name: "default".to_string(),
            layout_json: serde_json::to_string(&layout_value)
                .map_err(|err| Error::Unexpected(err.into()))?,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        let nodes = if is_system {
            build_theme_nodes(theme.id, &page_templates)?
        } else {
            Vec::new()
        };

        let initial_nodes: Vec<ThemeDraftNode> = page_templates
            .iter()
            .map(|page| ThemeDraftNode {
                node_key: page.key.clone(),
                blueprint: page.blueprint.clone(),
            })
            .collect();

        let version_payload = ThemeDraft {
            tokens: tokens_value.clone(),
            layout: layout_value.clone(),
            nodes: initial_nodes,
        };

        let version = ThemeVersion {
            id: Uuid::new_v4(),
            theme_id: theme.id,
            version_number: 1,
            status: "active".to_string(),
            snapshot_json: serde_json::to_string(&version_payload)
                .map_err(|err| Error::Unexpected(err.into()))?,
            created_at: "".to_string(),
        };

        let binding = ThemeBinding {
            id: Uuid::new_v4(),
            realm_id,
            client_id: None,
            theme_id: theme.id,
            active_version_id: version.id,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };

        let bundle = ThemeBundle {
            theme: &theme,
            tokens: &tokens,
            layout: &layout,
            nodes: &nodes,
            version: &version,
            binding: Some(&binding),
        };

        self.persist_theme_bundle(bundle, tx).await?;

        Ok(theme)
    }

    async fn persist_theme_bundle(
        &self,
        bundle: ThemeBundle<'_>,
        tx: &mut dyn Transaction,
    ) -> Result<()> {
        self.repo.create_theme(bundle.theme, Some(tx)).await?;
        self.repo.upsert_tokens(bundle.tokens, Some(tx)).await?;
        self.repo.upsert_layout(bundle.layout, Some(tx)).await?;
        for node in bundle.nodes {
            self.repo.upsert_node(node, Some(tx)).await?;
        }
        self.repo.create_version(bundle.version, Some(tx)).await?;
        if let Some(binding) = bundle.binding {
            self.repo.upsert_binding(binding, Some(tx)).await?;
        }
        Ok(())
    }

    async fn ensure_theme_pages(&self, theme_id: Uuid) -> Result<()> {
        let existing = self.repo.list_nodes(&theme_id).await?;
        let existing_keys: std::collections::HashSet<String> =
            existing.iter().map(|node| node.node_key.clone()).collect();
        let templates = theme_pages::system_pages();
        let missing: Vec<_> = templates
            .into_iter()
            .filter(|page| !existing_keys.contains(&page.key))
            .collect();

        if missing.is_empty() {
            return Ok(());
        }

        let nodes = build_theme_nodes(theme_id, &missing)?;
        for node in nodes {
            self.repo.upsert_node(&node, None).await?;
        }

        Ok(())
    }

    pub async fn resolve_binding(
        &self,
        realm_id: Uuid,
        client_id: Option<&str>,
    ) -> Result<Option<crate::domain::theme::ThemeBinding>> {
        let binding = if let Some(client_id) = client_id {
            self.repo.get_binding(&realm_id, Some(client_id)).await?
        } else {
            None
        };

        Ok(match binding {
            Some(binding) => Some(binding),
            None => self.repo.get_binding(&realm_id, None).await?,
        })
    }

    pub async fn get_asset(&self, theme_id: &Uuid, asset_id: &Uuid) -> Result<Option<ThemeAsset>> {
        self.repo.get_asset(theme_id, asset_id).await
    }

    pub async fn get_theme(&self, realm_id: Uuid, theme_id: &Uuid) -> Result<Option<Theme>> {
        self.repo.find_theme(&realm_id, theme_id).await
    }

    pub async fn list_themes(&self, realm_id: Uuid) -> Result<Vec<Theme>> {
        self.repo.list_themes(&realm_id).await
    }

    pub async fn list_versions(
        &self,
        theme_id: &Uuid,
    ) -> Result<Vec<crate::domain::theme::ThemeVersion>> {
        self.repo.list_versions(theme_id).await
    }

    pub async fn get_active_version_id(
        &self,
        realm_id: Uuid,
        theme_id: &Uuid,
    ) -> Result<Option<Uuid>> {
        let binding = self.resolve_binding(realm_id, None).await?;
        Ok(binding
            .filter(|binding| &binding.theme_id == theme_id)
            .map(|binding| binding.active_version_id))
    }

    pub async fn get_active_version_number(
        &self,
        realm_id: Uuid,
        theme_id: &Uuid,
    ) -> Result<Option<i64>> {
        let active_id = self.get_active_version_id(realm_id, theme_id).await?;
        if let Some(active_id) = active_id {
            if let Some(version) = self.repo.get_version(theme_id, &active_id).await? {
                return Ok(Some(version.version_number));
            }
        }
        Ok(None)
    }
}

fn parse_json(raw: &str, label: &str) -> Option<Value> {
    match serde_json::from_str::<Value>(raw) {
        Ok(value) => Some(value),
        Err(err) => {
            warn!("Failed to parse {} JSON: {}", label, err);
            None
        }
    }
}

fn parse_required_json(raw: &str, label: &str) -> Result<Value> {
    serde_json::from_str::<Value>(raw)
        .map_err(|err| Error::Validation(format!("Invalid {} JSON: {}", label, err)))
}

fn parse_version_payload(raw: &str) -> Option<ThemeDraft> {
    if let Ok(draft) = serde_json::from_str::<ThemeDraft>(raw) {
        return Some(draft);
    }

    if let Ok(snapshot) = serde_json::from_str::<ThemeSnapshot>(raw) {
        let blueprint = json!({
            "layout": "default",
            "blocks": snapshot.blocks,
        });
        return Some(ThemeDraft {
            tokens: snapshot.tokens,
            layout: snapshot.layout,
            nodes: vec![ThemeDraftNode {
                node_key: "login".to_string(),
                blueprint,
            }],
        });
    }

    None
}

fn parse_blueprint(raw: &str) -> Option<(Vec<ThemeBlock>, Option<String>)> {
    let value: Value = serde_json::from_str(raw).ok()?;
    parse_blueprint_value(value)
}

fn parse_blueprint_value(value: Value) -> Option<(Vec<ThemeBlock>, Option<String>)> {
    match value {
        Value::Array(items) => {
            let blocks: Vec<ThemeBlock> = serde_json::from_value(Value::Array(items)).ok()?;
            Some((blocks, None))
        }
        Value::Object(mut obj) => {
            let layout = obj
                .remove("layout")
                .and_then(|val| val.as_str().map(|s| s.to_string()));
            let blocks = obj
                .remove("blocks")
                .and_then(|val| serde_json::from_value::<Vec<ThemeBlock>>(val).ok())
                .unwrap_or_default();
            Some((blocks, layout))
        }
        _ => None,
    }
}

async fn resolve_layout(
    repo: &Arc<dyn ThemeRepository>,
    theme_id: Uuid,
    layout_hint: Option<String>,
) -> Result<Value> {
    if let Some(name) = layout_hint {
        if let Some(layout) = repo.get_layout(&theme_id, &name).await? {
            if let Some(parsed) = parse_json(&layout.layout_json, "theme_layout") {
                return Ok(parsed);
            }
        }
    }

    let default_layout_entry = repo.get_layout(&theme_id, "default").await?;
    if let Some(layout) = default_layout_entry {
        if let Some(parsed) = parse_json(&layout.layout_json, "theme_layout") {
            return Ok(parsed);
        }
    }

    let layouts = repo.list_layouts(&theme_id).await?;
    if let Some(layout) = layouts.first() {
        if let Some(parsed) = parse_json(&layout.layout_json, "theme_layout") {
            return Ok(parsed);
        }
    }

    Ok(default_layout())
}

fn default_tokens() -> Value {
    json!({
        "colors": {
            "primary": "var(--primary)",
            "background": "var(--background)",
            "text": "var(--foreground)",
            "surface": "var(--card)"
        },
        "appearance": {
            "mode": "auto"
        },
        "typography": {
            "font_family": "system-ui",
            "base_size": 16
        },
        "radius": {
            "base": 8
        }
    })
}

fn default_layout() -> Value {
    json!({
        "shell": "CenteredCard",
        "slots": ["main"]
    })
}

fn default_page_blocks(page_key: &str) -> (Vec<ThemeBlock>, Option<String>) {
    theme_pages::default_page_blueprint(page_key)
        .and_then(parse_blueprint_value)
        .unwrap_or_else(|| (Vec::new(), None))
}

fn build_theme_nodes(theme_id: Uuid, pages: &[ThemePageTemplate]) -> Result<Vec<ThemeNode>> {
    let mut nodes = Vec::with_capacity(pages.len());
    for page in pages {
        let blueprint_json =
            serde_json::to_string(&page.blueprint).map_err(|err| Error::Unexpected(err.into()))?;
        nodes.push(ThemeNode {
            id: Uuid::new_v4(),
            theme_id,
            node_key: page.key.clone(),
            blueprint_json,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        });
    }
    Ok(nodes)
}
