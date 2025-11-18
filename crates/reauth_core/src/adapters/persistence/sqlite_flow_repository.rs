use crate::adapters::persistence::connection::Database;
use crate::domain::auth_flow::LoginSession;
use crate::{
    domain::auth_flow::{AuthFlow, AuthFlowStep, AuthenticatorConfig},
    error::{Error, Result},
    ports::flow_repository::FlowRepository,
};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

pub struct SqliteFlowRepository {
    pool: Database,
}

impl SqliteFlowRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FlowRepository for SqliteFlowRepository {
    async fn find_flow_by_name(&self, realm_id: &Uuid, name: &str) -> Result<Option<AuthFlow>> {
        let flow = sqlx::query_as("SELECT * FROM auth_flows WHERE realm_id = ? AND name = ?")
            .bind(realm_id.to_string())
            .bind(name)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(flow)
    }

    async fn find_steps_for_flow(&self, flow_id: &Uuid) -> Result<Vec<AuthFlowStep>> {
        let steps =
            sqlx::query_as("SELECT * FROM auth_flow_steps WHERE flow_id = ? ORDER BY priority ASC")
                .bind(flow_id.to_string())
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(steps)
    }

    async fn find_config_for_authenticator(
        &self,
        realm_id: &Uuid,
        authenticator_name: &str,
    ) -> Result<Option<AuthenticatorConfig>> {
        let config = sqlx::query_as(
            "SELECT * FROM authenticator_config WHERE realm_id = ? AND authenticator_name = ?",
        )
        .bind(realm_id.to_string())
        .bind(authenticator_name)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(config)
    }

    async fn create_flow(&self, flow: &AuthFlow) -> Result<()> {
        sqlx::query("INSERT INTO auth_flows (id, realm_id, name) VALUES (?, ?, ?)")
            .bind(flow.id.to_string())
            .bind(flow.realm_id.to_string())
            .bind(&flow.name)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn add_step_to_flow(&self, step: &AuthFlowStep) -> Result<()> {
        sqlx::query(
            "INSERT INTO auth_flow_steps (id, flow_id, authenticator_name, priority) VALUES (?, ?, ?, ?)"
        )
            .bind(step.id.to_string())
            .bind(step.flow_id.to_string())
            .bind(&step.authenticator_name)
            .bind(step.priority)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn save_login_session(&self, session: &LoginSession) -> Result<()> {
        sqlx::query(
            "INSERT INTO login_sessions (id, realm_id, flow_id, current_step, user_id, state_data, expires_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
            .bind(session.id.to_string())
            .bind(session.realm_id.to_string())
            .bind(session.flow_id.to_string())
            .bind(session.current_step)
            .bind(session.user_id.map(|id| id.to_string()))
            .bind(&session.state_data)
            .bind(session.expires_at)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn find_login_session_by_id(&self, id: &Uuid) -> Result<Option<LoginSession>> {
        let session =
            sqlx::query_as("SELECT * FROM login_sessions WHERE id = ? AND expires_at > ?")
                .bind(id.to_string())
                .bind(Utc::now())
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(session)
    }

    async fn update_login_session(&self, session: &LoginSession) -> Result<()> {
        sqlx::query(
            "UPDATE login_sessions
             SET current_step = ?, user_id = ?, state_data = ?
             WHERE id = ?",
        )
        .bind(session.current_step)
        .bind(session.user_id.map(|id| id.to_string()))
        .bind(&session.state_data)
        .bind(session.id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn delete_login_session(&self, id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM login_sessions WHERE id = ?")
            .bind(id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }
}
