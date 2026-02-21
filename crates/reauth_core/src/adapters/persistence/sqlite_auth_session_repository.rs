use crate::adapters::persistence::connection::Database;
use crate::domain::auth_session::{AuthenticationSession, SessionStatus};
use crate::error::{Error, Result};
use crate::ports::auth_session_repository::AuthSessionRepository;
use async_trait::async_trait;
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

pub struct SqliteAuthSessionRepository {
    pool: Database,
}

impl SqliteAuthSessionRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthSessionRepository for SqliteAuthSessionRepository {
    async fn create(&self, session: &AuthenticationSession) -> Result<()> {
        sqlx::query(
            "INSERT INTO auth_sessions (
                id, realm_id, flow_version_id, current_node_id, context, status, created_at, expires_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(session.id.to_string())
            .bind(session.realm_id.to_string())
            .bind(session.flow_version_id.to_string())
            .bind(&session.current_node_id)
            .bind(sqlx::types::Json(&session.context))
            .bind(session.status.to_string())
            .bind(session.created_at)
            .bind(session.expires_at)
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<AuthenticationSession>> {
        // 1. Query into the intermediary Row struct
        let row = sqlx::query_as::<_, AuthSessionRow>("SELECT * FROM auth_sessions WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;

        // 2. Map Row -> Domain Object
        if let Some(r) = row {
            // Parse UUIDs manually to be safe
            let session_id = Uuid::parse_str(&r.id).unwrap_or_default();
            let realm_id = Uuid::parse_str(&r.realm_id).unwrap_or_default();
            let flow_version_id = Uuid::parse_str(&r.flow_version_id).unwrap_or_default();

            // Parse user_id if it exists
            let user_id = r
                .user_id
                .map(|uid_str| Uuid::parse_str(&uid_str).unwrap_or_default());

            let status_str = r.status.trim().to_lowercase();

            let status = match status_str.as_str() {
                "active" => SessionStatus::Active,
                "completed" => SessionStatus::Completed,
                "failed" => SessionStatus::Failed,
                _ => {
                    // Log the actual failing string so we can debug
                    error!(
                        "CRITICAL ERROR: DB Status '{}' could not be mapped. Defaulting to Failed.",
                        r.status
                    );
                    SessionStatus::Failed
                }
            };

            Ok(Some(AuthenticationSession {
                id: session_id,
                realm_id,
                flow_version_id,
                current_node_id: r.current_node_id,
                context: r.context.0, // Extract value from sqlx::types::Json wrapper
                status,
                user_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
                expires_at: r.expires_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update(&self, session: &AuthenticationSession) -> Result<()> {
        let query = "
            UPDATE auth_sessions
            SET
                realm_id = ?,
                flow_version_id = ?,
                current_node_id = ?,
                context = ?,
                status = ?,
                user_id = ?,
                updated_at = ?
            WHERE id = ?
        ";

        sqlx::query(query)
            .bind(session.realm_id.to_string())
            .bind(session.flow_version_id.to_string())
            .bind(&session.current_node_id)
            // Consistency Fix: Use sqlx::types::Json like you did in 'create'
            // instead of .to_string(), unless your column is strictly TEXT.
            .bind(sqlx::types::Json(&session.context))
            .bind(session.status.to_string())
            .bind(session.user_id.map(|id| id.to_string()))
            .bind(Utc::now())
            .bind(session.id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(anyhow::anyhow!("Failed to update session: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM auth_sessions WHERE id = ?")
            .bind(id.to_string())
            .execute(&*self.pool)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }
}

// Temporary struct to handle SQLite's TEXT vs UUID behavior safely
#[derive(sqlx::FromRow)]
struct AuthSessionRow {
    id: String,
    realm_id: String,
    flow_version_id: String,
    current_node_id: String,
    context: sqlx::types::Json<serde_json::Value>, // Handles the JSON text automatically
    status: String,
    user_id: Option<String>, // <--- Read as String first!
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    expires_at: chrono::DateTime<Utc>,
}
