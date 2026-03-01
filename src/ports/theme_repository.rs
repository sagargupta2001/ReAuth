use crate::domain::theme::{
    Theme, ThemeAsset, ThemeAssetMeta, ThemeBinding, ThemeLayout, ThemeNode, ThemeTokens,
    ThemeVersion,
};
use crate::error::Result;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ThemeRepository: Send + Sync {
    async fn create_theme(&self, theme: &Theme, tx: Option<&mut dyn Transaction>) -> Result<()>;
    async fn update_theme(&self, theme: &Theme, tx: Option<&mut dyn Transaction>) -> Result<()>;
    async fn set_theme_system(
        &self,
        theme_id: &Uuid,
        is_system: bool,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn delete_theme(
        &self,
        realm_id: &Uuid,
        theme_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn find_theme(&self, realm_id: &Uuid, theme_id: &Uuid) -> Result<Option<Theme>>;
    async fn list_themes(&self, realm_id: &Uuid) -> Result<Vec<Theme>>;

    async fn upsert_tokens(
        &self,
        tokens: &ThemeTokens,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn get_tokens(&self, theme_id: &Uuid) -> Result<Option<ThemeTokens>>;

    async fn upsert_layout(
        &self,
        layout: &ThemeLayout,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn get_layout(&self, theme_id: &Uuid, name: &str) -> Result<Option<ThemeLayout>>;
    async fn list_layouts(&self, theme_id: &Uuid) -> Result<Vec<ThemeLayout>>;

    async fn upsert_node(&self, node: &ThemeNode, tx: Option<&mut dyn Transaction>) -> Result<()>;
    async fn get_node(&self, theme_id: &Uuid, node_key: &str) -> Result<Option<ThemeNode>>;
    async fn list_nodes(&self, theme_id: &Uuid) -> Result<Vec<ThemeNode>>;
    async fn delete_node(
        &self,
        theme_id: &Uuid,
        node_key: &str,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;

    async fn create_asset(
        &self,
        asset: &ThemeAsset,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn get_asset(&self, theme_id: &Uuid, asset_id: &Uuid) -> Result<Option<ThemeAsset>>;
    async fn list_assets(&self, theme_id: &Uuid) -> Result<Vec<ThemeAssetMeta>>;
    async fn delete_asset(
        &self,
        theme_id: &Uuid,
        asset_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;

    async fn create_version(
        &self,
        version: &ThemeVersion,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn get_version(&self, theme_id: &Uuid, version_id: &Uuid)
        -> Result<Option<ThemeVersion>>;
    async fn list_versions(&self, theme_id: &Uuid) -> Result<Vec<ThemeVersion>>;
    async fn set_version_status(
        &self,
        version_id: &Uuid,
        status: &str,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;

    async fn upsert_binding(
        &self,
        binding: &ThemeBinding,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
    async fn get_binding(
        &self,
        realm_id: &Uuid,
        client_id: Option<&str>,
    ) -> Result<Option<ThemeBinding>>;
    async fn list_bindings(&self, realm_id: &Uuid) -> Result<Vec<ThemeBinding>>;
    async fn delete_binding(
        &self,
        realm_id: &Uuid,
        client_id: Option<&str>,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()>;
}
