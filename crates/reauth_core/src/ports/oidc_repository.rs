use crate::{
    domain::oidc::{AuthCode, OidcClient},
    error::Result,
};
use async_trait::async_trait;

#[async_trait]
pub trait OidcRepository: Send + Sync {
    // Client Management
    async fn find_client_by_id(&self, client_id: &str) -> Result<Option<OidcClient>>;
    async fn create_client(&self, client: &OidcClient) -> Result<()>;

    // Auth Code Management
    async fn save_auth_code(&self, code: &AuthCode) -> Result<()>;
    async fn find_auth_code_by_code(&self, code: &str) -> Result<Option<AuthCode>>;
    async fn delete_auth_code(&self, code: &str) -> Result<()>;
}
