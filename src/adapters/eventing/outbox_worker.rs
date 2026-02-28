use crate::adapters::observability::telemetry_store::TelemetryDatabase;
use crate::adapters::persistence::connection::Database;
use anyhow::anyhow;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rand::RngExt;
use serde::Deserialize;
use sqlx::Row;
use std::error::Error as StdError;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;

const BACKOFF_SCHEDULE_SECS: [i64; 5] = [60, 300, 1800, 7200, 43200];
const BACKOFF_JITTER_FRACTION: f64 = 0.2;
const MAX_CONSECUTIVE_FAILURES: i64 = 10;

#[derive(Debug)]
struct OutboxRow {
    id: String,
    realm_id: Option<String>,
    event_type: String,
    event_version: String,
    payload_json: String,
    attempt_count: i64,
}

pub struct OutboxWorker {
    db: Database,
    telemetry_db: TelemetryDatabase,
    http_client: reqwest::Client,
    poll_interval: Duration,
    batch_size: i64,
    worker_id: String,
}

struct WebhookFailureLog<'a> {
    outbox: &'a OutboxRow,
    target: &'a WebhookTarget,
    attempt: i64,
    response_status: Option<i64>,
    response_body: Option<&'a str>,
    error: Option<&'a str>,
    error_chain: Option<&'a [String]>,
    latency_ms: i64,
}

impl OutboxWorker {
    pub fn new(db: Database, telemetry_db: TelemetryDatabase) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            db,
            telemetry_db,
            http_client,
            poll_interval: Duration::from_millis(500),
            batch_size: 50,
            worker_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn spawn(self) {
        tokio::spawn(async move {
            info!("Outbox worker started");
            let mut ticker = tokio::time::interval(self.poll_interval);
            loop {
                ticker.tick().await;
                if let Err(err) = self.process_batch().await {
                    error!("Outbox worker batch failed: {}", err);
                }
            }
        });
    }

    async fn process_batch(&self) -> anyhow::Result<()> {
        let rows = sqlx::query(
            "SELECT id, realm_id, event_type, event_version, payload_json, attempt_count
             FROM event_outbox
             WHERE status IN ('pending', 'retry')
               AND (next_attempt_at IS NULL OR datetime(next_attempt_at) <= datetime('now'))
               AND (locked_at IS NULL OR locked_at <= datetime('now','-5 minutes'))
             ORDER BY occurred_at
             LIMIT ?",
        )
        .bind(self.batch_size)
        .fetch_all(&*self.db)
        .await?;

        for row in rows {
            let outbox = OutboxRow {
                id: row.try_get("id")?,
                realm_id: row.try_get("realm_id")?,
                event_type: row.try_get("event_type")?,
                event_version: row.try_get("event_version")?,
                payload_json: row.try_get("payload_json")?,
                attempt_count: row.try_get("attempt_count")?,
            };

            let claimed = sqlx::query(
                "UPDATE event_outbox
                 SET status = 'processing', locked_at = CURRENT_TIMESTAMP, locked_by = ?
                 WHERE id = ? AND status IN ('pending', 'retry')",
            )
            .bind(&self.worker_id)
            .bind(&outbox.id)
            .execute(&*self.db)
            .await?
            .rows_affected();

            if claimed == 0 {
                continue;
            }

            if let Err(err) = self.handle_event(&outbox).await {
                error!("Failed processing outbox event {}: {}", outbox.id, err);
                let attempt = outbox.attempt_count + 1;
                let last_error = format!("{}", err);
                if let Some(next_attempt) = next_attempt_at(attempt) {
                    sqlx::query(
                        "UPDATE event_outbox
                         SET status = 'retry', attempt_count = ?, next_attempt_at = ?, last_error = ?, locked_at = NULL, locked_by = NULL
                         WHERE id = ?",
                    )
                    .bind(attempt)
                    .bind(next_attempt.to_rfc3339())
                    .bind(last_error)
                    .bind(&outbox.id)
                    .execute(&*self.db)
                    .await?;
                } else {
                    sqlx::query(
                        "UPDATE event_outbox
                         SET status = 'dead', attempt_count = ?, last_error = ?, locked_at = NULL, locked_by = NULL
                         WHERE id = ?",
                    )
                    .bind(attempt)
                    .bind(last_error)
                    .bind(&outbox.id)
                    .execute(&*self.db)
                    .await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_event(&self, outbox: &OutboxRow) -> anyhow::Result<()> {
        let attempt = outbox.attempt_count + 1;
        let mut failures: Vec<String> = Vec::new();

        let webhook_targets = self.fetch_webhook_targets(outbox).await?;

        if webhook_targets.is_empty() {
            self.log_delivery(DeliveryLogEntry {
                outbox,
                target_type: "none",
                target_id: "none",
                attempt,
                response_status: None,
                response_body: None,
                error: Some("no_targets".to_string()),
                error_chain: None,
                latency_ms: 0,
            })
            .await?;

            sqlx::query(
                "UPDATE event_outbox
                 SET status = 'skipped', attempt_count = ?, last_error = ?, locked_at = NULL, locked_by = NULL
                 WHERE id = ?",
            )
            .bind(attempt)
            .bind("no_targets")
            .bind(&outbox.id)
            .execute(&*self.db)
            .await?;

            return Ok(());
        }

        for target in webhook_targets {
            match self.dispatch_webhook(outbox, attempt, &target).await {
                Ok(()) => {}
                Err(err) => failures.push(err.to_string()),
            }
        }

        if failures.is_empty() {
            sqlx::query(
                "UPDATE event_outbox
                 SET status = 'delivered', attempt_count = ?, last_error = NULL, locked_at = NULL, locked_by = NULL
                 WHERE id = ?",
            )
            .bind(attempt)
            .bind(&outbox.id)
            .execute(&*self.db)
            .await?;
        } else {
            let last_error = failures
                .last()
                .cloned()
                .unwrap_or_else(|| "delivery_failed".to_string());
            if let Some(next_attempt) = next_attempt_at(attempt) {
                sqlx::query(
                    "UPDATE event_outbox
                     SET status = 'retry', attempt_count = ?, next_attempt_at = ?, last_error = ?, locked_at = NULL, locked_by = NULL
                     WHERE id = ?",
                )
                .bind(attempt)
                .bind(next_attempt.to_rfc3339())
                .bind(last_error)
                .bind(&outbox.id)
                .execute(&*self.db)
                .await?;
            } else {
                sqlx::query(
                    "UPDATE event_outbox
                     SET status = 'dead', attempt_count = ?, last_error = ?, locked_at = NULL, locked_by = NULL
                     WHERE id = ?",
                )
                .bind(attempt)
                .bind(last_error)
                .bind(&outbox.id)
                .execute(&*self.db)
                .await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, sqlx::FromRow)]
struct WebhookTargetRow {
    id: String,
    url: String,
    http_method: String,
    signing_secret: String,
    custom_headers: String,
}

struct DeliveryLogEntry<'a> {
    outbox: &'a OutboxRow,
    target_type: &'a str,
    target_id: &'a str,
    attempt: i64,
    response_status: Option<i64>,
    response_body: Option<String>,
    error: Option<String>,
    error_chain: Option<String>,
    latency_ms: i64,
}

#[derive(Debug, Clone)]
struct WebhookTarget {
    id: String,
    url: String,
    http_method: String,
    signing_secret: String,
    headers: Vec<(String, String)>,
}

impl OutboxWorker {
    async fn fetch_webhook_targets(
        &self,
        outbox: &OutboxRow,
    ) -> anyhow::Result<Vec<WebhookTarget>> {
        let Some(realm_id) = outbox.realm_id.as_ref() else {
            return Ok(Vec::new());
        };

        let rows: Vec<WebhookTargetRow> = sqlx::query_as(
            r#"
            SELECT e.id, e.url, e.http_method, e.signing_secret, e.custom_headers
            FROM webhook_endpoints e
            JOIN webhook_subscriptions s ON s.endpoint_id = e.id
            WHERE e.status = 'active'
              AND s.enabled = 1
              AND s.event_type = ?
              AND e.realm_id = ?
        "#,
        )
        .bind(&outbox.event_type)
        .bind(realm_id)
        .fetch_all(&*self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| WebhookTarget {
                id: row.id,
                url: row.url,
                http_method: row.http_method,
                signing_secret: row.signing_secret,
                headers: parse_custom_headers(&row.custom_headers),
            })
            .collect())
    }

    async fn dispatch_webhook(
        &self,
        outbox: &OutboxRow,
        attempt: i64,
        target: &WebhookTarget,
    ) -> anyhow::Result<()> {
        let start = Instant::now();
        let signature = sign_payload(&target.signing_secret, &outbox.payload_json);

        let method = parse_http_method(&target.http_method);
        let mut request = self
            .http_client
            .request(method, &target.url)
            .header("Content-Type", "application/json")
            .header("Reauth-Event-Id", &outbox.id)
            .header("Reauth-Event-Type", &outbox.event_type)
            .header("Reauth-Event-Version", &outbox.event_version)
            .header("Reauth-Signature", signature);

        for (key, value) in &target.headers {
            request = request.header(key, value);
        }

        let response = request.body(outbox.payload_json.clone()).send().await;
        let latency_ms = start.elapsed().as_millis() as i64;

        match response {
            Ok(resp) => {
                let status_code = resp.status().as_u16() as i64;
                let body = resp.text().await.unwrap_or_default();
                let is_success = (200..300).contains(&status_code);
                let error = if is_success {
                    None
                } else {
                    Some(format!("http_{}", status_code))
                };

                self.log_delivery(DeliveryLogEntry {
                    outbox,
                    target_type: "webhook",
                    target_id: &target.id,
                    attempt,
                    response_status: Some(status_code),
                    response_body: Some(body.clone()),
                    error: error.clone(),
                    error_chain: None,
                    latency_ms,
                })
                .await?;

                if !is_success {
                    let log = WebhookFailureLog {
                        outbox,
                        target,
                        attempt,
                        response_status: Some(status_code),
                        response_body: Some(&body),
                        error: error.as_deref(),
                        error_chain: None,
                        latency_ms,
                    };
                    if let Err(err) = self.log_webhook_failure_telemetry(log).await {
                        warn!("Failed to log webhook failure telemetry: {}", err);
                    }

                    self.record_webhook_failure(&target.id, error.as_deref().unwrap_or(""))
                        .await?;
                    return Err(anyhow!(
                        "webhook {} failed with status {}",
                        target.id,
                        status_code
                    ));
                }

                self.record_webhook_success(&target.id).await?;
            }
            Err(err) => {
                let error = err.to_string();
                let error_chain = collect_error_chain(&err);
                let error_chain_json = serialize_error_chain(&error_chain);
                self.log_delivery(DeliveryLogEntry {
                    outbox,
                    target_type: "webhook",
                    target_id: &target.id,
                    attempt,
                    response_status: None,
                    response_body: None,
                    error: Some(error.clone()),
                    error_chain: error_chain_json.clone(),
                    latency_ms,
                })
                .await?;
                let error_chain_ref = (!error_chain.is_empty()).then_some(error_chain.as_slice());
                let log = WebhookFailureLog {
                    outbox,
                    target,
                    attempt,
                    response_status: None,
                    response_body: None,
                    error: Some(&error),
                    error_chain: error_chain_ref,
                    latency_ms,
                };
                if let Err(err) = self.log_webhook_failure_telemetry(log).await {
                    warn!("Failed to log webhook failure telemetry: {}", err);
                }
                self.record_webhook_failure(&target.id, &error).await?;
                return Err(err.into());
            }
        }

        Ok(())
    }

    async fn record_webhook_success(&self, endpoint_id: &str) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE webhook_endpoints
             SET consecutive_failures = 0, last_failure_at = NULL, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(endpoint_id)
        .execute(&*self.db)
        .await?;
        Ok(())
    }

    async fn record_webhook_failure(&self, endpoint_id: &str, reason: &str) -> anyhow::Result<()> {
        let row = sqlx::query(
            "SELECT consecutive_failures
             FROM webhook_endpoints
             WHERE id = ?",
        )
        .bind(endpoint_id)
        .fetch_optional(&*self.db)
        .await?;

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
            .await?;
        } else {
            sqlx::query(
                "UPDATE webhook_endpoints
                 SET consecutive_failures = ?, last_failure_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?",
            )
            .bind(next_failures)
            .bind(endpoint_id)
            .execute(&*self.db)
            .await?;
        }

        Ok(())
    }

    async fn log_webhook_failure_telemetry(
        &self,
        entry: WebhookFailureLog<'_>,
    ) -> anyhow::Result<()> {
        let fields = serde_json::json!({
            "event_id": entry.outbox.id,
            "event_type": entry.outbox.event_type,
            "event_version": entry.outbox.event_version,
            "attempt": entry.attempt,
            "target_id": entry.target.id,
            "url": entry.target.url,
            "response_status": entry.response_status,
            "response_body": entry.response_body,
            "error": entry.error,
            "error_chain": entry.error_chain.map(|chain| chain.to_vec()),
        });
        let fields_json = serde_json::to_string(&fields).unwrap_or_else(|_| "{}".to_string());
        let message = if entry.response_status.is_some() {
            "webhook.response.non_2xx"
        } else {
            "webhook.request.failed"
        };

        sqlx::query(
            "INSERT INTO telemetry_logs (
                id, timestamp, level, target, message, fields, request_id, trace_id, span_id,
                parent_id, user_id, realm, method, route, path, status, duration_ms
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(Utc::now().to_rfc3339())
        .bind("ERROR")
        .bind("webhook.delivery")
        .bind(message)
        .bind(fields_json)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(entry.outbox.realm_id.clone())
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(entry.response_status)
        .bind(Some(entry.latency_ms))
        .execute(&*self.telemetry_db)
        .await?;

        Ok(())
    }

    async fn log_delivery(&self, entry: DeliveryLogEntry<'_>) -> anyhow::Result<()> {
        let delivered_at = Utc::now().to_rfc3339();
        let delivery_id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO delivery_logs (
                id, event_id, realm_id, target_type, target_id, event_type, event_version, attempt,
                payload, payload_compressed, response_status, response_body, error, error_chain, latency_ms, delivered_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(delivery_id)
        .bind(&entry.outbox.id)
        .bind(entry.outbox.realm_id.as_ref())
        .bind(entry.target_type)
        .bind(entry.target_id)
        .bind(&entry.outbox.event_type)
        .bind(&entry.outbox.event_version)
        .bind(entry.attempt)
        .bind(&entry.outbox.payload_json)
        .bind(false)
        .bind(entry.response_status)
        .bind(entry.response_body)
        .bind(entry.error)
        .bind(entry.error_chain)
        .bind(entry.latency_ms)
        .bind(delivered_at)
        .execute(&*self.telemetry_db)
        .await?;

        Ok(())
    }
}

fn parse_custom_headers(raw: &str) -> Vec<(String, String)> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };

    match value.as_object() {
        Some(map) => map
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect(),
        None => Vec::new(),
    }
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

fn next_attempt_at(attempt: i64) -> Option<DateTime<Utc>> {
    let idx = if attempt <= 1 {
        0
    } else {
        (attempt - 1) as usize
    };
    if idx >= BACKOFF_SCHEDULE_SECS.len() {
        return None;
    }

    let base = BACKOFF_SCHEDULE_SECS[idx] as f64;
    let jitter = base * BACKOFF_JITTER_FRACTION;
    let min = (base - jitter).max(1.0);
    let max = base + jitter;
    let secs = rand::rng().random_range(min..=max).round() as i64;

    Some(Utc::now() + ChronoDuration::seconds(secs))
}

fn collect_error_chain(error: &reqwest::Error) -> Vec<String> {
    let mut chain = Vec::new();
    let mut current: Option<&(dyn StdError + 'static)> = Some(error);
    while let Some(err) = current {
        chain.push(err.to_string());
        current = err.source();
    }
    chain
}

fn serialize_error_chain(chain: &[String]) -> Option<String> {
    if chain.is_empty() {
        return None;
    }
    serde_json::to_string(chain).ok()
}

fn parse_http_method(method: &str) -> reqwest::Method {
    reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::POST)
}
