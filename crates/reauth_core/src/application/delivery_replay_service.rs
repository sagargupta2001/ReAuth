use crate::adapters::observability::telemetry_store::TelemetryDatabase;
use crate::adapters::persistence::connection::Database;
use crate::application::telemetry_service::TelemetryService;
use crate::application::webhook_service::WebhookService;
use crate::domain::telemetry::DeliveryLog;
use crate::error::{Error, Result};
use anyhow::anyhow;
use chrono::Utc;
use manager::grpc::plugin::v1::event_listener_client::EventListenerClient;
use manager::grpc::plugin::v1::EventRequest;
use manager::PluginManager;
use serde::Serialize;
use serde_json::Value;
use sqlx::Row;
use std::time::{Duration, Instant};
use uuid::Uuid;

const MAX_CONSECUTIVE_FAILURES: i64 = 10;
const PLUGIN_LEGACY_VERSION: &str = "v0";

#[derive(Debug, Serialize)]
pub struct ReplayDeliveryResult {
    pub delivery_id: String,
    pub target_type: String,
    pub target_id: String,
    pub response_status: Option<i64>,
    pub error: Option<String>,
    pub latency_ms: Option<i64>,
}

pub struct DeliveryReplayService {
    telemetry_service: std::sync::Arc<TelemetryService>,
    webhook_service: std::sync::Arc<WebhookService>,
    telemetry_db: TelemetryDatabase,
    db: Database,
    plugin_manager: PluginManager,
    http_client: reqwest::Client,
}

impl DeliveryReplayService {
    pub fn new(
        telemetry_service: std::sync::Arc<TelemetryService>,
        webhook_service: std::sync::Arc<WebhookService>,
        telemetry_db: TelemetryDatabase,
        db: Database,
        plugin_manager: PluginManager,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            telemetry_service,
            webhook_service,
            telemetry_db,
            db,
            plugin_manager,
            http_client,
        }
    }

    pub async fn replay_delivery(&self, delivery_id: &str) -> Result<ReplayDeliveryResult> {
        let Some(log) = self.telemetry_service.get_delivery_log(delivery_id).await? else {
            return Err(Error::NotFound("Delivery log not found".to_string()));
        };

        match log.target_type.as_str() {
            "webhook" => self.replay_webhook(log).await,
            "plugin" => self.replay_plugin(log).await,
            _ => Err(Error::Validation(format!(
                "Unsupported target type {}",
                log.target_type
            ))),
        }
    }

    async fn replay_webhook(&self, log: DeliveryLog) -> Result<ReplayDeliveryResult> {
        if log.payload_compressed {
            return Err(Error::Validation(
                "Compressed payload replay not supported".to_string(),
            ));
        }

        let realm_id = log
            .realm_id
            .ok_or_else(|| Error::Validation("Missing realm_id".to_string()))?;
        let endpoint_id = Uuid::parse_str(&log.target_id)?;
        let details = self
            .webhook_service
            .get_endpoint(realm_id, endpoint_id)
            .await?;
        let endpoint = details.endpoint;

        let signature = sign_payload(&endpoint.signing_secret, &log.payload);
        let start = Instant::now();
        let mut request = self
            .http_client
            .post(&endpoint.url)
            .header("Content-Type", "application/json")
            .header("Reauth-Event-Id", &log.event_id)
            .header("Reauth-Event-Type", &log.event_type)
            .header("Reauth-Event-Version", &log.event_version)
            .header("Reauth-Signature", signature);

        for (key, value) in &endpoint.custom_headers {
            request = request.header(key, value);
        }

        let response = request.body(log.payload.clone()).send().await;
        let latency_ms = start.elapsed().as_millis() as i64;

        match response {
            Ok(resp) => {
                let status_code = resp.status().as_u16() as i64;
                let body = resp.text().await.unwrap_or_default();
                let error = if status_code >= 500 {
                    Some(format!("http_{}", status_code))
                } else {
                    None
                };

                let new_delivery_id = self
                    .insert_delivery_log(
                        &log,
                        Some(status_code),
                        Some(body.clone()),
                        error.clone(),
                        latency_ms,
                    )
                    .await?;

                if error.is_some() {
                    self.record_webhook_failure(
                        &endpoint_id.to_string(),
                        error.as_deref().unwrap_or(""),
                    )
                    .await?;
                } else {
                    self.record_webhook_success(&endpoint_id.to_string())
                        .await?;
                }

                Ok(ReplayDeliveryResult {
                    delivery_id: new_delivery_id,
                    target_type: log.target_type,
                    target_id: log.target_id,
                    response_status: Some(status_code),
                    error,
                    latency_ms: Some(latency_ms),
                })
            }
            Err(err) => {
                let error = err.to_string();
                let new_delivery_id = self
                    .insert_delivery_log(&log, None, None, Some(error.clone()), latency_ms)
                    .await?;
                self.record_webhook_failure(&endpoint_id.to_string(), &error)
                    .await?;

                Ok(ReplayDeliveryResult {
                    delivery_id: new_delivery_id,
                    target_type: log.target_type,
                    target_id: log.target_id,
                    response_status: None,
                    error: Some(error),
                    latency_ms: Some(latency_ms),
                })
            }
        }
    }

    async fn replay_plugin(&self, log: DeliveryLog) -> Result<ReplayDeliveryResult> {
        if log.payload_compressed {
            return Err(Error::Validation(
                "Compressed payload replay not supported".to_string(),
            ));
        }

        let active_plugins = self.plugin_manager.get_all_active_plugins().await;
        let Some((manifest, channel)) = active_plugins
            .into_iter()
            .find(|(manifest, _)| manifest.id == log.target_id)
        else {
            return Err(Error::System("Plugin is not active".to_string()));
        };

        let payload_json = match map_payload_for_version(
            &log.payload,
            &log.event_version,
            &manifest.events.supported_event_version,
        ) {
            Ok(payload) => payload,
            Err(err) => {
                let error = format!("version_mismatch: {}", err);
                let new_delivery_id = self
                    .insert_delivery_log(&log, None, None, Some(error.clone()), 0)
                    .await?;
                return Ok(ReplayDeliveryResult {
                    delivery_id: new_delivery_id,
                    target_type: log.target_type,
                    target_id: log.target_id,
                    response_status: None,
                    error: Some(error),
                    latency_ms: Some(0),
                });
            }
        };

        let start = Instant::now();
        let mut client = EventListenerClient::new(channel.clone());
        let request = tonic::Request::new(EventRequest {
            event_type: log.event_type.clone(),
            event_payload_json: payload_json,
        });

        let result = tokio::time::timeout(Duration::from_secs(5), client.on_event(request)).await;
        let latency_ms = start.elapsed().as_millis() as i64;

        match result {
            Ok(Ok(_)) => {
                let new_delivery_id = self
                    .insert_delivery_log(&log, None, None, None, latency_ms)
                    .await?;
                Ok(ReplayDeliveryResult {
                    delivery_id: new_delivery_id,
                    target_type: log.target_type,
                    target_id: log.target_id,
                    response_status: None,
                    error: None,
                    latency_ms: Some(latency_ms),
                })
            }
            Ok(Err(err)) => {
                let error = err.to_string();
                let new_delivery_id = self
                    .insert_delivery_log(&log, None, None, Some(error.clone()), latency_ms)
                    .await?;
                Ok(ReplayDeliveryResult {
                    delivery_id: new_delivery_id,
                    target_type: log.target_type,
                    target_id: log.target_id,
                    response_status: None,
                    error: Some(error),
                    latency_ms: Some(latency_ms),
                })
            }
            Err(err) => {
                let error = format!("timeout: {}", err);
                let new_delivery_id = self
                    .insert_delivery_log(&log, None, None, Some(error.clone()), latency_ms)
                    .await?;
                Ok(ReplayDeliveryResult {
                    delivery_id: new_delivery_id,
                    target_type: log.target_type,
                    target_id: log.target_id,
                    response_status: None,
                    error: Some(error),
                    latency_ms: Some(latency_ms),
                })
            }
        }
    }

    async fn insert_delivery_log(
        &self,
        log: &DeliveryLog,
        response_status: Option<i64>,
        response_body: Option<String>,
        error: Option<String>,
        latency_ms: i64,
    ) -> Result<String> {
        let delivered_at = Utc::now().to_rfc3339();
        let delivery_id = Uuid::new_v4().to_string();
        let attempt = log.attempt + 1;

        sqlx::query(
            "INSERT INTO delivery_logs (
                id, event_id, realm_id, target_type, target_id, event_type, event_version, attempt,
                payload, payload_compressed, response_status, response_body, error, latency_ms, delivered_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&delivery_id)
        .bind(&log.event_id)
        .bind(log.realm_id.map(|id| id.to_string()))
        .bind(&log.target_type)
        .bind(&log.target_id)
        .bind(&log.event_type)
        .bind(&log.event_version)
        .bind(attempt)
        .bind(&log.payload)
        .bind(log.payload_compressed)
        .bind(response_status)
        .bind(response_body)
        .bind(error)
        .bind(latency_ms)
        .bind(delivered_at)
        .execute(&*self.telemetry_db)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        Ok(delivery_id)
    }

    async fn record_webhook_success(&self, endpoint_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE webhook_endpoints
             SET consecutive_failures = 0, last_failure_at = NULL, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(endpoint_id)
        .execute(&*self.db)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }

    async fn record_webhook_failure(&self, endpoint_id: &str, reason: &str) -> Result<()> {
        let row = sqlx::query(
            "SELECT consecutive_failures
             FROM webhook_endpoints
             WHERE id = ?",
        )
        .bind(endpoint_id)
        .fetch_optional(&*self.db)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

        let Some(row) = row else {
            return Ok(());
        };

        let failures: i64 = row.get("consecutive_failures");
        let next_failures = failures + 1;

        if next_failures >= MAX_CONSECUTIVE_FAILURES {
            sqlx::query(
                "UPDATE webhook_endpoints
                 SET consecutive_failures = ?, status = ?, disabled_at = CURRENT_TIMESTAMP,
                     disabled_reason = ?, last_failure_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?",
            )
            .bind(next_failures)
            .bind("disabled_system")
            .bind(reason)
            .bind(endpoint_id)
            .execute(&*self.db)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        } else {
            sqlx::query(
                "UPDATE webhook_endpoints
                 SET consecutive_failures = ?, last_failure_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?",
            )
            .bind(next_failures)
            .bind(endpoint_id)
            .execute(&*self.db)
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        }

        Ok(())
    }
}

fn map_payload_for_version(
    payload_json: &str,
    outbox_version: &str,
    supported_version: &str,
) -> anyhow::Result<String> {
    if supported_version == outbox_version {
        return Ok(payload_json.to_string());
    }

    if supported_version == PLUGIN_LEGACY_VERSION {
        let value: Value = serde_json::from_str(payload_json)?;
        if let Some(data) = value.get("data") {
            return Ok(data.to_string());
        }
        return Ok("{}".to_string());
    }

    Err(anyhow!(
        "unsupported mapping from {} to {}",
        outbox_version,
        supported_version
    ))
}

fn sign_payload(secret: &str, payload: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .unwrap_or_else(|_| Hmac::<Sha256>::new_from_slice(b"").unwrap());
    mac.update(payload.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(result)
}
