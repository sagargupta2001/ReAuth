use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::transaction::SqliteTransaction;
use crate::domain::theme::{
    Theme, ThemeAsset, ThemeAssetMeta, ThemeBinding, ThemeLayout, ThemeNode, ThemeTokens,
    ThemeVersion,
};
use crate::error::{Error, Result};
use crate::ports::theme_repository::ThemeRepository;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use sqlx::Row;
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteThemeRepository {
    pool: Database,
}

impl SqliteThemeRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }

    fn parse_uuid(value: &str, fallback: &Uuid) -> Uuid {
        Uuid::parse_str(value).unwrap_or(*fallback)
    }
}

#[async_trait]
impl ThemeRepository for SqliteThemeRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "themes", db_op = "insert")
    )]
    async fn create_theme(&self, theme: &Theme, tx: Option<&mut dyn Transaction>) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO themes (id, realm_id, name, description, is_system) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(theme.id.to_string())
        .bind(theme.realm_id.to_string())
        .bind(&theme.name)
        .bind(&theme.description)
        .bind(theme.is_system);

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "themes", db_op = "update")
    )]
    async fn update_theme(&self, theme: &Theme, tx: Option<&mut dyn Transaction>) -> Result<()> {
        let query = sqlx::query(
            "UPDATE themes SET name = ?, description = ?, flow_binding_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND realm_id = ?",
        )
        .bind(&theme.name)
        .bind(&theme.description)
        .bind(&theme.flow_binding_id)
        .bind(theme.id.to_string())
        .bind(theme.realm_id.to_string());

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "themes", db_op = "update")
    )]
    async fn set_theme_system(
        &self,
        theme_id: &Uuid,
        is_system: bool,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query("UPDATE themes SET is_system = ? WHERE id = ?")
            .bind(is_system)
            .bind(theme_id.to_string());

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "themes", db_op = "delete")
    )]
    async fn delete_theme(
        &self,
        realm_id: &Uuid,
        theme_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query =
            sqlx::query("DELETE FROM themes WHERE id = ? AND realm_id = ? AND is_system = 0")
                .bind(theme_id.to_string())
                .bind(realm_id.to_string());

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "themes", db_op = "select")
    )]
    async fn find_theme(&self, realm_id: &Uuid, theme_id: &Uuid) -> Result<Option<Theme>> {
        let row = sqlx::query(
            "SELECT id, realm_id, name, description, flow_binding_id, is_system, created_at, updated_at FROM themes WHERE id = ? AND realm_id = ?",
        )
        .bind(theme_id.to_string())
        .bind(realm_id.to_string())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row.map(|row| Theme {
            id: Self::parse_uuid(row.get::<String, _>("id").as_str(), theme_id),
            realm_id: Self::parse_uuid(row.get::<String, _>("realm_id").as_str(), realm_id),
            name: row.get("name"),
            description: row.get("description"),
            flow_binding_id: row.get("flow_binding_id"),
            is_system: row.get("is_system"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "themes", db_op = "select")
    )]
    async fn find_theme_with_tx(
        &self,
        realm_id: &Uuid,
        theme_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<Option<Theme>> {
        let query = sqlx::query(
            "SELECT id, realm_id, name, description, flow_binding_id, is_system, created_at, updated_at FROM themes WHERE id = ? AND realm_id = ?",
        )
        .bind(theme_id.to_string())
        .bind(realm_id.to_string());

        let row = if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX");
            query.fetch_optional(&mut **sql_tx).await
        } else {
            query.fetch_optional(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row.map(|row| Theme {
            id: Self::parse_uuid(row.get::<String, _>("id").as_str(), theme_id),
            realm_id: Self::parse_uuid(row.get::<String, _>("realm_id").as_str(), realm_id),
            name: row.get("name"),
            description: row.get("description"),
            flow_binding_id: row.get("flow_binding_id"),
            is_system: row.get("is_system"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "themes", db_op = "select")
    )]
    async fn list_themes(&self, realm_id: &Uuid) -> Result<Vec<Theme>> {
        let rows = sqlx::query(
            "SELECT id, realm_id, name, description, flow_binding_id, is_system, created_at, updated_at FROM themes WHERE realm_id = ? ORDER BY created_at DESC",
        )
        .bind(realm_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| Theme {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                    .unwrap_or_else(|_| Uuid::new_v4()),
                realm_id: *realm_id,
                name: row.get("name"),
                description: row.get("description"),
                flow_binding_id: row.get("flow_binding_id"),
                is_system: row.get("is_system"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_tokens", db_op = "upsert")
    )]
    async fn upsert_tokens(
        &self,
        tokens: &ThemeTokens,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO theme_tokens (id, theme_id, tokens_json)
             VALUES (?, ?, ?)
             ON CONFLICT(theme_id)
             DO UPDATE SET tokens_json = excluded.tokens_json, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(tokens.id.to_string())
        .bind(tokens.theme_id.to_string())
        .bind(&tokens.tokens_json);

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_tokens", db_op = "select")
    )]
    async fn get_tokens(&self, theme_id: &Uuid) -> Result<Option<ThemeTokens>> {
        let row = sqlx::query(
            "SELECT id, theme_id, tokens_json, created_at, updated_at FROM theme_tokens WHERE theme_id = ?",
        )
        .bind(theme_id.to_string())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row.map(|row| ThemeTokens {
            id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                .unwrap_or_else(|_| Uuid::new_v4()),
            theme_id: *theme_id,
            tokens_json: row.get("tokens_json"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_layouts", db_op = "upsert")
    )]
    async fn upsert_layout(
        &self,
        layout: &ThemeLayout,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO theme_layouts (id, theme_id, name, layout_json)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(theme_id, name)
             DO UPDATE SET layout_json = excluded.layout_json, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(layout.id.to_string())
        .bind(layout.theme_id.to_string())
        .bind(&layout.name)
        .bind(&layout.layout_json);

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_layouts", db_op = "select")
    )]
    async fn get_layout(&self, theme_id: &Uuid, name: &str) -> Result<Option<ThemeLayout>> {
        let row = sqlx::query(
            "SELECT id, theme_id, name, layout_json, created_at, updated_at FROM theme_layouts WHERE theme_id = ? AND name = ?",
        )
        .bind(theme_id.to_string())
        .bind(name)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row.map(|row| ThemeLayout {
            id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                .unwrap_or_else(|_| Uuid::new_v4()),
            theme_id: *theme_id,
            name: row.get("name"),
            layout_json: row.get("layout_json"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_layouts", db_op = "select")
    )]
    async fn list_layouts(&self, theme_id: &Uuid) -> Result<Vec<ThemeLayout>> {
        let rows = sqlx::query(
            "SELECT id, theme_id, name, layout_json, created_at, updated_at FROM theme_layouts WHERE theme_id = ? ORDER BY name ASC",
        )
        .bind(theme_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| ThemeLayout {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                    .unwrap_or_else(|_| Uuid::new_v4()),
                theme_id: *theme_id,
                name: row.get("name"),
                layout_json: row.get("layout_json"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_nodes", db_op = "upsert")
    )]
    async fn upsert_node(&self, node: &ThemeNode, tx: Option<&mut dyn Transaction>) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO theme_nodes (id, theme_id, node_key, blueprint_json)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(theme_id, node_key)
             DO UPDATE SET blueprint_json = excluded.blueprint_json, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(node.id.to_string())
        .bind(node.theme_id.to_string())
        .bind(&node.node_key)
        .bind(&node.blueprint_json);

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_nodes", db_op = "select")
    )]
    async fn get_node(&self, theme_id: &Uuid, node_key: &str) -> Result<Option<ThemeNode>> {
        let row = sqlx::query(
            "SELECT id, theme_id, node_key, blueprint_json, created_at, updated_at FROM theme_nodes WHERE theme_id = ? AND node_key = ?",
        )
        .bind(theme_id.to_string())
        .bind(node_key)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row.map(|row| ThemeNode {
            id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                .unwrap_or_else(|_| Uuid::new_v4()),
            theme_id: *theme_id,
            node_key: row.get("node_key"),
            blueprint_json: row.get("blueprint_json"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_nodes", db_op = "select")
    )]
    async fn list_nodes(&self, theme_id: &Uuid) -> Result<Vec<ThemeNode>> {
        let rows = sqlx::query(
            "SELECT id, theme_id, node_key, blueprint_json, created_at, updated_at FROM theme_nodes WHERE theme_id = ? ORDER BY node_key ASC",
        )
        .bind(theme_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| ThemeNode {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                    .unwrap_or_else(|_| Uuid::new_v4()),
                theme_id: *theme_id,
                node_key: row.get("node_key"),
                blueprint_json: row.get("blueprint_json"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_nodes", db_op = "delete")
    )]
    async fn delete_node(
        &self,
        theme_id: &Uuid,
        node_key: &str,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query("DELETE FROM theme_nodes WHERE theme_id = ? AND node_key = ?")
            .bind(theme_id.to_string())
            .bind(node_key);

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }

        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_assets", db_op = "insert")
    )]
    async fn create_asset(
        &self,
        asset: &ThemeAsset,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO theme_assets (id, theme_id, asset_type, filename, mime_type, byte_size, checksum, data)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(asset.id.to_string())
        .bind(asset.theme_id.to_string())
        .bind(&asset.asset_type)
        .bind(&asset.filename)
        .bind(&asset.mime_type)
        .bind(asset.byte_size)
        .bind(&asset.checksum)
        .bind(&asset.data);

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_assets", db_op = "select")
    )]
    async fn get_asset(&self, theme_id: &Uuid, asset_id: &Uuid) -> Result<Option<ThemeAsset>> {
        let row = sqlx::query(
            "SELECT id, theme_id, asset_type, filename, mime_type, byte_size, checksum, data, created_at, updated_at
             FROM theme_assets WHERE theme_id = ? AND id = ?",
        )
        .bind(theme_id.to_string())
        .bind(asset_id.to_string())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row.map(|row| ThemeAsset {
            id: Self::parse_uuid(row.get::<String, _>("id").as_str(), asset_id),
            theme_id: Self::parse_uuid(row.get::<String, _>("theme_id").as_str(), theme_id),
            asset_type: row.get("asset_type"),
            filename: row.get("filename"),
            mime_type: row.get("mime_type"),
            byte_size: row.get("byte_size"),
            checksum: row.get("checksum"),
            data: row.get("data"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_assets", db_op = "select")
    )]
    async fn list_assets(&self, theme_id: &Uuid) -> Result<Vec<ThemeAssetMeta>> {
        let rows = sqlx::query(
            "SELECT id, theme_id, asset_type, filename, mime_type, byte_size, checksum, created_at, updated_at
             FROM theme_assets WHERE theme_id = ? ORDER BY created_at DESC",
        )
        .bind(theme_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| ThemeAssetMeta {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                    .unwrap_or_else(|_| Uuid::new_v4()),
                theme_id: *theme_id,
                asset_type: row.get("asset_type"),
                filename: row.get("filename"),
                mime_type: row.get("mime_type"),
                byte_size: row.get("byte_size"),
                checksum: row.get("checksum"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_assets", db_op = "delete")
    )]
    async fn delete_asset(
        &self,
        theme_id: &Uuid,
        asset_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query("DELETE FROM theme_assets WHERE theme_id = ? AND id = ?")
            .bind(theme_id.to_string())
            .bind(asset_id.to_string());

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    async fn set_draft_exists(
        &self,
        theme_id: &Uuid,
        exists: bool,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO theme_draft_meta (theme_id, draft_exists) VALUES (?, ?)
             ON CONFLICT(theme_id) DO UPDATE SET draft_exists = excluded.draft_exists, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(theme_id.to_string())
        .bind(exists);

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    async fn get_draft_exists(&self, theme_id: &Uuid) -> Result<bool> {
        let row = sqlx::query("SELECT draft_exists FROM theme_draft_meta WHERE theme_id = ?")
            .bind(theme_id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row
            .and_then(|row| row.try_get::<i64, _>("draft_exists").ok())
            .map(|value| value != 0)
            .unwrap_or(false))
    }

    async fn get_draft_exists_with_tx(
        &self,
        theme_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<bool> {
        let query = sqlx::query("SELECT draft_exists FROM theme_draft_meta WHERE theme_id = ?")
            .bind(theme_id.to_string());

        let row = if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX");
            query.fetch_optional(&mut **sql_tx).await
        } else {
            query.fetch_optional(&*self.pool).await
        }
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row
            .and_then(|row| row.try_get::<i64, _>("draft_exists").ok())
            .map(|value| value != 0)
            .unwrap_or(false))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_versions", db_op = "insert")
    )]
    async fn create_version(
        &self,
        version: &ThemeVersion,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO theme_versions (id, theme_id, version_number, status, snapshot_json)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(version.id.to_string())
        .bind(version.theme_id.to_string())
        .bind(version.version_number)
        .bind(&version.status)
        .bind(&version.snapshot_json);

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_versions", db_op = "select")
    )]
    async fn get_version(
        &self,
        theme_id: &Uuid,
        version_id: &Uuid,
    ) -> Result<Option<ThemeVersion>> {
        let row = sqlx::query(
            "SELECT id, theme_id, version_number, status, snapshot_json, created_at
             FROM theme_versions WHERE theme_id = ? AND id = ?",
        )
        .bind(theme_id.to_string())
        .bind(version_id.to_string())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row.map(|row| ThemeVersion {
            id: Self::parse_uuid(row.get::<String, _>("id").as_str(), version_id),
            theme_id: *theme_id,
            version_number: row.get("version_number"),
            status: row.get("status"),
            snapshot_json: row.get("snapshot_json"),
            created_at: row.get("created_at"),
        }))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_versions", db_op = "select")
    )]
    async fn list_versions(&self, theme_id: &Uuid) -> Result<Vec<ThemeVersion>> {
        let rows = sqlx::query(
            "SELECT id, theme_id, version_number, status, snapshot_json, created_at
             FROM theme_versions WHERE theme_id = ? ORDER BY version_number DESC",
        )
        .bind(theme_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| ThemeVersion {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                    .unwrap_or_else(|_| Uuid::new_v4()),
                theme_id: *theme_id,
                version_number: row.get("version_number"),
                status: row.get("status"),
                snapshot_json: row.get("snapshot_json"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_versions", db_op = "update")
    )]
    async fn set_version_status(
        &self,
        version_id: &Uuid,
        status: &str,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query("UPDATE theme_versions SET status = ? WHERE id = ?")
            .bind(status)
            .bind(version_id.to_string());

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_bindings", db_op = "upsert")
    )]
    async fn upsert_binding(
        &self,
        binding: &ThemeBinding,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let mut tx = tx;
        let rows_affected = if let Some(client_id) = binding.client_id.as_deref() {
            let query = sqlx::query(
                "UPDATE theme_bindings
                 SET theme_id = ?, active_version_id = ?, updated_at = CURRENT_TIMESTAMP
                 WHERE realm_id = ? AND client_id = ?",
            )
            .bind(binding.theme_id.to_string())
            .bind(binding.active_version_id.to_string())
            .bind(binding.realm_id.to_string())
            .bind(client_id);

            if let Some(tx) = tx.as_mut() {
                let sql_tx = SqliteTransaction::from_trait(&mut **tx).expect("Invalid TX type");
                query
                    .execute(&mut **sql_tx)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?
                    .rows_affected()
            } else {
                query
                    .execute(&*self.pool)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?
                    .rows_affected()
            }
        } else {
            let query = sqlx::query(
                "UPDATE theme_bindings
                 SET theme_id = ?, active_version_id = ?, updated_at = CURRENT_TIMESTAMP
                 WHERE realm_id = ? AND client_id IS NULL",
            )
            .bind(binding.theme_id.to_string())
            .bind(binding.active_version_id.to_string())
            .bind(binding.realm_id.to_string());

            if let Some(tx) = tx.as_deref_mut() {
                let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
                query
                    .execute(&mut **sql_tx)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?
                    .rows_affected()
            } else {
                query
                    .execute(&*self.pool)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?
                    .rows_affected()
            }
        };

        if rows_affected == 0 {
            let insert = sqlx::query(
                "INSERT INTO theme_bindings (id, realm_id, client_id, theme_id, active_version_id)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(binding.id.to_string())
            .bind(binding.realm_id.to_string())
            .bind(&binding.client_id)
            .bind(binding.theme_id.to_string())
            .bind(binding.active_version_id.to_string());

            if let Some(tx) = tx.as_mut() {
                let sql_tx = SqliteTransaction::from_trait(&mut **tx).expect("Invalid TX type");
                insert
                    .execute(&mut **sql_tx)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            } else {
                insert
                    .execute(&*self.pool)
                    .await
                    .map_err(|e| Error::Unexpected(e.into()))?;
            }
        }
        Ok(())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_bindings", db_op = "select")
    )]
    async fn get_binding(
        &self,
        realm_id: &Uuid,
        client_id: Option<&str>,
    ) -> Result<Option<ThemeBinding>> {
        let row = if let Some(client_id) = client_id {
            sqlx::query(
                "SELECT id, realm_id, client_id, theme_id, active_version_id, created_at, updated_at
                 FROM theme_bindings WHERE realm_id = ? AND client_id = ?",
            )
            .bind(realm_id.to_string())
            .bind(client_id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?
        } else {
            sqlx::query(
                "SELECT id, realm_id, client_id, theme_id, active_version_id, created_at, updated_at
                 FROM theme_bindings WHERE realm_id = ? AND client_id IS NULL",
            )
            .bind(realm_id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?
        };

        Ok(row.map(|row| ThemeBinding {
            id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                .unwrap_or_else(|_| Uuid::new_v4()),
            realm_id: *realm_id,
            client_id: row.get("client_id"),
            theme_id: Uuid::parse_str(row.get::<String, _>("theme_id").as_str())
                .unwrap_or_else(|_| Uuid::new_v4()),
            active_version_id: Uuid::parse_str(row.get::<String, _>("active_version_id").as_str())
                .unwrap_or_else(|_| Uuid::new_v4()),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_bindings", db_op = "select")
    )]
    async fn get_binding_with_tx(
        &self,
        realm_id: &Uuid,
        client_id: Option<&str>,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<Option<ThemeBinding>> {
        let row = if let Some(client_id) = client_id {
            let query = sqlx::query(
                "SELECT id, realm_id, client_id, theme_id, active_version_id, created_at, updated_at
                 FROM theme_bindings WHERE realm_id = ? AND client_id = ?",
            )
            .bind(realm_id.to_string())
            .bind(client_id);

            if let Some(tx) = tx {
                let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX");
                query.fetch_optional(&mut **sql_tx).await
            } else {
                query.fetch_optional(&*self.pool).await
            }
            .map_err(|e| Error::Unexpected(e.into()))?
        } else {
            let query = sqlx::query(
                "SELECT id, realm_id, client_id, theme_id, active_version_id, created_at, updated_at
                 FROM theme_bindings WHERE realm_id = ? AND client_id IS NULL",
            )
            .bind(realm_id.to_string());

            if let Some(tx) = tx {
                let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX");
                query.fetch_optional(&mut **sql_tx).await
            } else {
                query.fetch_optional(&*self.pool).await
            }
            .map_err(|e| Error::Unexpected(e.into()))?
        };

        Ok(row.map(|row| ThemeBinding {
            id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                .unwrap_or_else(|_| Uuid::new_v4()),
            realm_id: *realm_id,
            client_id: row.get("client_id"),
            theme_id: Uuid::parse_str(row.get::<String, _>("theme_id").as_str())
                .unwrap_or_else(|_| Uuid::new_v4()),
            active_version_id: Uuid::parse_str(row.get::<String, _>("active_version_id").as_str())
                .unwrap_or_else(|_| Uuid::new_v4()),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_bindings", db_op = "select")
    )]
    async fn list_bindings(&self, realm_id: &Uuid) -> Result<Vec<ThemeBinding>> {
        let rows = sqlx::query(
            "SELECT id, realm_id, client_id, theme_id, active_version_id, created_at, updated_at
             FROM theme_bindings WHERE realm_id = ? ORDER BY created_at DESC",
        )
        .bind(realm_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| ThemeBinding {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                    .unwrap_or_else(|_| Uuid::new_v4()),
                realm_id: *realm_id,
                client_id: row.get("client_id"),
                theme_id: Uuid::parse_str(row.get::<String, _>("theme_id").as_str())
                    .unwrap_or_else(|_| Uuid::new_v4()),
                active_version_id: Uuid::parse_str(
                    row.get::<String, _>("active_version_id").as_str(),
                )
                .unwrap_or_else(|_| Uuid::new_v4()),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "theme_bindings", db_op = "delete")
    )]
    async fn delete_binding(
        &self,
        realm_id: &Uuid,
        client_id: Option<&str>,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = if let Some(client_id) = client_id {
            sqlx::query("DELETE FROM theme_bindings WHERE realm_id = ? AND client_id = ?")
                .bind(realm_id.to_string())
                .bind(client_id)
        } else {
            sqlx::query("DELETE FROM theme_bindings WHERE realm_id = ? AND client_id IS NULL")
                .bind(realm_id.to_string())
        };

        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            query
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            query
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
        Ok(())
    }
}
