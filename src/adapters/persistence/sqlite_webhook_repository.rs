use crate::adapters::persistence::connection::Database;
use crate::adapters::persistence::transaction::SqliteTransaction;
use crate::domain::webhook::{WebhookEndpoint, WebhookSubscription};
use crate::error::{Error, Result};
use crate::ports::transaction_manager::Transaction;
use crate::ports::webhook_repository::WebhookRepository;
use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

pub struct SqliteWebhookRepository {
    pool: Database,
}

impl SqliteWebhookRepository {
    pub fn new(pool: Database) -> Self {
        Self { pool }
    }

    fn parse_headers(raw: &str) -> HashMap<String, String> {
        serde_json::from_str::<HashMap<String, String>>(raw).unwrap_or_default()
    }

    fn serialize_headers(headers: &HashMap<String, String>) -> String {
        serde_json::to_string(headers).unwrap_or_else(|_| "{}".to_string())
    }
}

#[async_trait]
impl WebhookRepository for SqliteWebhookRepository {
    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "webhook_endpoints", db_op = "insert")
    )]
    async fn create_endpoint(
        &self,
        endpoint: &WebhookEndpoint,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO webhook_endpoints (
                id, realm_id, name, url, http_method, status, signing_secret, custom_headers, description
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(endpoint.id.to_string())
        .bind(endpoint.realm_id.to_string())
        .bind(&endpoint.name)
        .bind(&endpoint.url)
        .bind(&endpoint.http_method)
        .bind(&endpoint.status)
        .bind(&endpoint.signing_secret)
        .bind(Self::serialize_headers(&endpoint.custom_headers))
        .bind(&endpoint.description);

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
        fields(telemetry = "span", db_table = "webhook_endpoints", db_op = "update")
    )]
    async fn update_endpoint(
        &self,
        endpoint: &WebhookEndpoint,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query(
            "UPDATE webhook_endpoints
             SET name = ?, url = ?, http_method = ?, status = ?, signing_secret = ?, custom_headers = ?, description = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ? AND realm_id = ?",
        )
        .bind(&endpoint.name)
        .bind(&endpoint.url)
        .bind(&endpoint.http_method)
        .bind(&endpoint.status)
        .bind(&endpoint.signing_secret)
        .bind(Self::serialize_headers(&endpoint.custom_headers))
        .bind(&endpoint.description)
        .bind(endpoint.id.to_string())
        .bind(endpoint.realm_id.to_string());

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
        fields(telemetry = "span", db_table = "webhook_endpoints", db_op = "delete")
    )]
    async fn delete_endpoint(
        &self,
        realm_id: &Uuid,
        endpoint_id: &Uuid,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = sqlx::query("DELETE FROM webhook_endpoints WHERE id = ? AND realm_id = ?")
            .bind(endpoint_id.to_string())
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
        fields(telemetry = "span", db_table = "webhook_endpoints", db_op = "update")
    )]
    async fn set_endpoint_status(
        &self,
        realm_id: &Uuid,
        endpoint_id: &Uuid,
        status: &str,
        reason: Option<&str>,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let query = if status == "active" {
            sqlx::query(
                "UPDATE webhook_endpoints
                 SET status = ?, disabled_at = NULL, disabled_reason = NULL,
                     consecutive_failures = 0, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ? AND realm_id = ?",
            )
            .bind(status)
            .bind(endpoint_id.to_string())
            .bind(realm_id.to_string())
        } else {
            sqlx::query(
                "UPDATE webhook_endpoints
                 SET status = ?, disabled_at = CURRENT_TIMESTAMP, disabled_reason = ?,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = ? AND realm_id = ?",
            )
            .bind(status)
            .bind(reason)
            .bind(endpoint_id.to_string())
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

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "webhook_endpoints", db_op = "select")
    )]
    async fn find_endpoint(
        &self,
        realm_id: &Uuid,
        endpoint_id: &Uuid,
    ) -> Result<Option<WebhookEndpoint>> {
        let row = sqlx::query(
            "SELECT id, realm_id, name, url, http_method, status, signing_secret, custom_headers, description,
                    consecutive_failures, last_fired_at, last_failure_at, disabled_at, disabled_reason,
                    created_at, updated_at
             FROM webhook_endpoints
             WHERE id = ? AND realm_id = ?",
        )
        .bind(endpoint_id.to_string())
        .bind(realm_id.to_string())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(row.map(|row| WebhookEndpoint {
            id: Uuid::parse_str(row.get::<String, _>("id").as_str()).unwrap_or(*endpoint_id),
            realm_id: Uuid::parse_str(row.get::<String, _>("realm_id").as_str())
                .unwrap_or(*realm_id),
            name: row.get("name"),
            url: row.get("url"),
            http_method: row.get("http_method"),
            status: row.get("status"),
            signing_secret: row.get("signing_secret"),
            custom_headers: Self::parse_headers(&row.get::<String, _>("custom_headers")),
            description: row.get("description"),
            consecutive_failures: row.get("consecutive_failures"),
            last_fired_at: row.get("last_fired_at"),
            last_failure_at: row.get("last_failure_at"),
            disabled_at: row.get("disabled_at"),
            disabled_reason: row.get("disabled_reason"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "webhook_endpoints", db_op = "select")
    )]
    async fn list_endpoints(&self, realm_id: &Uuid) -> Result<Vec<WebhookEndpoint>> {
        let rows = sqlx::query(
            "SELECT id, realm_id, name, url, http_method, status, signing_secret, custom_headers, description,
                    consecutive_failures, last_fired_at, last_failure_at, disabled_at, disabled_reason,
                    created_at, updated_at
             FROM webhook_endpoints
             WHERE realm_id = ?
             ORDER BY created_at DESC",
        )
        .bind(realm_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| WebhookEndpoint {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                    .unwrap_or_else(|_| Uuid::nil()),
                realm_id: Uuid::parse_str(row.get::<String, _>("realm_id").as_str())
                    .unwrap_or(*realm_id),
                name: row.get("name"),
                url: row.get("url"),
                http_method: row.get("http_method"),
                status: row.get("status"),
                signing_secret: row.get("signing_secret"),
                custom_headers: Self::parse_headers(&row.get::<String, _>("custom_headers")),
                description: row.get("description"),
                consecutive_failures: row.get("consecutive_failures"),
                last_fired_at: row.get("last_fired_at"),
                last_failure_at: row.get("last_failure_at"),
                disabled_at: row.get("disabled_at"),
                disabled_reason: row.get("disabled_reason"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", db_table = "webhook_endpoints", db_op = "select")
    )]
    async fn search_endpoints(
        &self,
        realm_id: &Uuid,
        query: &str,
        limit: i64,
    ) -> Result<Vec<WebhookEndpoint>> {
        let pattern = format!("%{}%", query.to_lowercase());
        let rows = sqlx::query(
            "SELECT id, realm_id, name, url, http_method, status, signing_secret, custom_headers, description,
                    consecutive_failures, last_fired_at, last_failure_at, disabled_at, disabled_reason,
                    created_at, updated_at
             FROM webhook_endpoints
             WHERE realm_id = ?
               AND (lower(name) LIKE ? OR lower(url) LIKE ?)
             ORDER BY updated_at DESC
             LIMIT ?",
        )
        .bind(realm_id.to_string())
        .bind(&pattern)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| WebhookEndpoint {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str())
                    .unwrap_or_else(|_| Uuid::nil()),
                realm_id: Uuid::parse_str(row.get::<String, _>("realm_id").as_str())
                    .unwrap_or(*realm_id),
                name: row.get("name"),
                url: row.get("url"),
                http_method: row.get("http_method"),
                status: row.get("status"),
                signing_secret: row.get("signing_secret"),
                custom_headers: Self::parse_headers(&row.get::<String, _>("custom_headers")),
                description: row.get("description"),
                consecutive_failures: row.get("consecutive_failures"),
                last_fired_at: row.get("last_fired_at"),
                last_failure_at: row.get("last_failure_at"),
                disabled_at: row.get("disabled_at"),
                disabled_reason: row.get("disabled_reason"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "webhook_subscriptions",
            db_op = "replace"
        )
    )]
    async fn upsert_subscriptions(
        &self,
        endpoint_id: &Uuid,
        event_types: &[String],
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        if let Some(tx) = tx {
            let sql_tx = SqliteTransaction::from_trait(tx).expect("Invalid TX type");
            for event_type in event_types {
                sqlx::query(
                    "INSERT INTO webhook_subscriptions (endpoint_id, event_type, enabled)
                     VALUES (?, ?, 1)
                     ON CONFLICT(endpoint_id, event_type) DO UPDATE SET enabled = excluded.enabled",
                )
                .bind(endpoint_id.to_string())
                .bind(event_type)
                .execute(&mut **sql_tx)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
            }
        } else {
            for event_type in event_types {
                sqlx::query(
                    "INSERT INTO webhook_subscriptions (endpoint_id, event_type, enabled)
                     VALUES (?, ?, 1)
                     ON CONFLICT(endpoint_id, event_type) DO UPDATE SET enabled = excluded.enabled",
                )
                .bind(endpoint_id.to_string())
                .bind(event_type)
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Unexpected(e.into()))?;
            }
        }

        Ok(())
    }

    #[instrument(
        skip_all,
        fields(
            telemetry = "span",
            db_table = "webhook_subscriptions",
            db_op = "upsert"
        )
    )]
    async fn set_subscription_enabled(
        &self,
        endpoint_id: &Uuid,
        event_type: &str,
        enabled: bool,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<()> {
        let enabled_flag = if enabled { 1 } else { 0 };
        let query = sqlx::query(
            "INSERT INTO webhook_subscriptions (endpoint_id, event_type, enabled)
             VALUES (?, ?, ?)
             ON CONFLICT(endpoint_id, event_type) DO UPDATE SET enabled = excluded.enabled",
        )
        .bind(endpoint_id.to_string())
        .bind(event_type)
        .bind(enabled_flag);

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
        fields(
            telemetry = "span",
            db_table = "webhook_subscriptions",
            db_op = "select"
        )
    )]
    async fn list_subscriptions(&self, endpoint_id: &Uuid) -> Result<Vec<WebhookSubscription>> {
        let rows = sqlx::query(
            "SELECT endpoint_id, event_type, enabled, created_at
             FROM webhook_subscriptions
             WHERE endpoint_id = ?
             ORDER BY event_type ",
        )
        .bind(endpoint_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|row| WebhookSubscription {
                endpoint_id: Uuid::parse_str(row.get::<String, _>("endpoint_id").as_str())
                    .unwrap_or(*endpoint_id),
                event_type: row.get("event_type"),
                enabled: row.get::<i64, _>("enabled") == 1,
                created_at: row.get("created_at"),
            })
            .collect())
    }
}
