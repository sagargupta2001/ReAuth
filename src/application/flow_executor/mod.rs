use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::audit_service::AuditService;
use crate::application::email_delivery_service::{
    EmailDeliveryService, RecoveryEmail, VerificationEmail,
};
use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::audit::NewAuditEvent;
use crate::domain::auth_session::{AuthenticationSession, SessionStatus};
use crate::domain::auth_session_action::AuthSessionAction;
use crate::domain::execution::lifecycle::NodeOutcome;
use crate::domain::execution::{ExecutionPlan, ExecutionResult, StepType};
use crate::error::{Error, Result};
use crate::ports::auth_session_action_repository::AuthSessionActionRepository;
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::flow_store::FlowStore;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::{info_span, instrument, Instrument};

pub struct FlowExecutor {
    session_repo: Arc<dyn AuthSessionRepository>,
    flow_store: Arc<dyn FlowStore>,
    registry: Arc<RuntimeRegistry>,
    action_repo: Arc<dyn AuthSessionActionRepository>,
    email_delivery: Option<Arc<EmailDeliveryService>>,
    audit_service: Option<Arc<AuditService>>,
}

impl FlowExecutor {
    pub fn new(
        session_repo: Arc<dyn AuthSessionRepository>,
        flow_store: Arc<dyn FlowStore>,
        registry: Arc<RuntimeRegistry>,
        action_repo: Arc<dyn AuthSessionActionRepository>,
        email_delivery: Option<Arc<EmailDeliveryService>>,
        audit_service: Option<Arc<AuditService>>,
    ) -> Self {
        Self {
            session_repo,
            flow_store,
            registry,
            action_repo,
            email_delivery,
            audit_service,
        }
    }

    #[instrument(skip_all, fields(telemetry = "span"))]
    pub async fn execute(
        &self,
        session_id: Uuid,
        mut user_input: Option<Value>,
    ) -> Result<ExecutionResult> {
        let mut session = self
            .session_repo
            .find_by_id(&session_id)
            .await?
            .ok_or(Error::NotFound("Session not found".into()))?;

        if session.status != SessionStatus::Active {
            self.heal_session(&mut session).await?;
            user_input = None;
        }

        if let Some(result) = self.pending_action_result(&session, user_input.is_some())? {
            return Ok(result);
        }

        let version = self
            .flow_store
            .get_version(&session.flow_version_id)
            .await?
            .ok_or(Error::System("Flow version missing".into()))?;

        let plan: ExecutionPlan = serde_json::from_str(&version.execution_artifact)
            .map_err(|e| Error::System(format!("Corrupt artifact: {}", e)))?;

        if let Some(input) = user_input {
            let current_node_def = plan
                .nodes
                .get(&session.current_node_id)
                .ok_or(Error::System("Current node not found in plan".into()))?;

            if current_node_def.step_type == StepType::Authenticator {
                let worker_key = current_node_def
                    .config
                    .get("auth_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("core.auth.password");

                let worker = self
                    .registry
                    .get_node(worker_key)
                    .ok_or(Error::System(format!("Worker not found: {}", worker_key)))?;

                let node_id = session.current_node_id.clone();
                let step_type = format!("{:?}", current_node_def.step_type);
                let handle_span = info_span!(
                    "flow.node.handle_input",
                    telemetry = "span",
                    node_id = %node_id,
                    step_type = %step_type,
                    worker = %worker_key
                );

                let previous_config = inject_node_config(&mut session, &current_node_def.config);
                let outcome = worker
                    .handle_input(&mut session, input)
                    .instrument(handle_span)
                    .await?;

                match outcome {
                    NodeOutcome::Continue { output } => {
                        // If DB is missing the link, we force it for the password node
                        let forced_next =
                            if session.current_node_id == "auth-password" && output == "success" {
                                Some("success".to_string())
                            } else {
                                None
                            };

                        let next_id = current_node_def
                            .next
                            .get(&output)
                            .or(forced_next.as_ref()) // <--- Use Patch if DB fails
                            .or_else(|| current_node_def.next.get("default"))
                            .ok_or_else(|| {
                                Error::Validation(
                                    "Flow Error: No path forward from this input".into(),
                                )
                            })?;
                        // ------------------------------------

                        let exit_span = info_span!(
                            "flow.node.on_exit",
                            telemetry = "span",
                            node_id = %node_id,
                            step_type = %step_type,
                            worker = %worker_key
                        );
                        worker.on_exit(&mut session).instrument(exit_span).await?;
                        restore_node_config(&mut session, previous_config);
                        session.current_node_id = next_id.clone();
                    }
                    NodeOutcome::Reject { .. } => {
                        let exec_span = info_span!(
                            "flow.node.execute",
                            telemetry = "span",
                            node_id = %node_id,
                            step_type = %step_type,
                            worker = %worker_key
                        );
                        let ui_outcome = worker.execute(&mut session).instrument(exec_span).await?;
                        restore_node_config(&mut session, previous_config);
                        self.session_repo.update(&session).await?;
                        if let NodeOutcome::SuspendForUI { screen, context } = ui_outcome {
                            let template_key = resolve_template_key(&current_node_def.config);
                            let context = attach_template_key(context, template_key.as_deref());
                            return Ok(ExecutionResult::Challenge {
                                screen_id: screen,
                                context,
                            });
                        }
                        return Err(Error::System("Rejecting node did not return UI".into()));
                    }
                    NodeOutcome::SuspendForAsync {
                        action_type,
                        token,
                        expires_at,
                        resume_node_id,
                        payload,
                        screen,
                        context,
                    } => {
                        restore_node_config(&mut session, previous_config);
                        let template_key = resolve_template_key(&current_node_def.config);
                        let context = attach_template_key(context, template_key.as_deref());
                        let result = self
                            .handle_async_suspend(
                                &mut session,
                                AsyncSuspendRequest {
                                    action_type,
                                    token,
                                    expires_at,
                                    resume_node_id,
                                    payload,
                                    screen,
                                    context,
                                },
                            )
                            .await?;
                        return Ok(result);
                    }
                    _ => {
                        restore_node_config(&mut session, previous_config);
                        return Err(Error::System("Unexpected outcome from handle_input".into()));
                    }
                }
            }
        }

        // 2. Main Execution Loop
        loop {
            let node_def = plan
                .nodes
                .get(&session.current_node_id)
                .ok_or(Error::System("Node missing from graph".into()))?;

            let node_id = session.current_node_id.clone();
            let step_type = format!("{:?}", node_def.step_type);

            let worker = match node_def.step_type {
                StepType::Authenticator => {
                    let key = node_def
                        .config
                        .get("auth_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("core.auth.password");
                    self.registry.get_node(key)
                }
                StepType::Logic => node_def
                    .config
                    .get("logic_type")
                    .and_then(|v| v.as_str())
                    .and_then(|key| self.registry.get_node(key)),
                _ => None,
            };

            if let Some(worker) = worker {
                let previous_config = inject_node_config(&mut session, &node_def.config);
                let enter_span = info_span!(
                    "flow.node.on_enter",
                    telemetry = "span",
                    node_id = %node_id,
                    step_type = %step_type
                );
                worker.on_enter(&mut session).instrument(enter_span).await?;
                let exec_span = info_span!(
                    "flow.node.execute",
                    telemetry = "span",
                    node_id = %node_id,
                    step_type = %step_type
                );
                let outcome = worker.execute(&mut session).instrument(exec_span).await?;

                match outcome {
                    NodeOutcome::Continue { output } => {
                        let available_keys: Vec<&String> = node_def.next.keys().collect();

                        let next_id = node_def
                            .next
                            .get(&output)
                            .or_else(|| node_def.next.get("default"))
                            .ok_or(Error::Validation(format!(
                                "Flow validation failed: Node '{}' returned output '{}' but allowed paths are {:?}",
                                session.current_node_id, output, available_keys
                            )))?;

                        let exit_span = info_span!(
                            "flow.node.on_exit",
                            telemetry = "span",
                            node_id = %node_id,
                            step_type = %step_type
                        );
                        worker.on_exit(&mut session).instrument(exit_span).await?;
                        restore_node_config(&mut session, previous_config);
                        session.current_node_id = next_id.clone();
                    }
                    NodeOutcome::SuspendForUI { screen, context } => {
                        let template_key = resolve_template_key(&node_def.config);
                        let context = attach_template_key(context, template_key.as_deref());
                        restore_node_config(&mut session, previous_config);
                        self.session_repo.update(&session).await?;
                        return Ok(ExecutionResult::Challenge {
                            screen_id: screen,
                            context,
                        });
                    }
                    NodeOutcome::SuspendForAsync {
                        action_type,
                        token,
                        expires_at,
                        resume_node_id,
                        payload,
                        screen,
                        context,
                    } => {
                        restore_node_config(&mut session, previous_config);
                        let template_key = resolve_template_key(&node_def.config);
                        let context = attach_template_key(context, template_key.as_deref());
                        let result = self
                            .handle_async_suspend(
                                &mut session,
                                AsyncSuspendRequest {
                                    action_type,
                                    token,
                                    expires_at,
                                    resume_node_id,
                                    payload,
                                    screen,
                                    context,
                                },
                            )
                            .await?;
                        return Ok(result);
                    }
                    NodeOutcome::FlowSuccess { user_id: _ } => {
                        session.status = SessionStatus::Completed;
                        restore_node_config(&mut session, previous_config);
                        self.session_repo.update(&session).await?;
                        return Ok(ExecutionResult::Success {
                            redirect_url: "/".to_string(),
                        });
                    }
                    NodeOutcome::FlowFailure { reason } => {
                        session.status = SessionStatus::Failed;
                        restore_node_config(&mut session, previous_config);
                        self.session_repo.update(&session).await?;
                        return Ok(ExecutionResult::Failure { reason });
                    }
                    _ => {
                        restore_node_config(&mut session, previous_config);
                        return Err(Error::System("Unhandled execution outcome".into()));
                    }
                }
            } else {
                match node_def.step_type {
                    StepType::Logic => {
                        let logic_span = info_span!(
                            "flow.node.logic",
                            telemetry = "span",
                            node_id = %node_id,
                            step_type = %step_type
                        );
                        let _guard = logic_span.enter();
                        let next_id = if is_condition_node(&node_def.config) {
                            let outcome = evaluate_condition(&session.context, &node_def.config)?;
                            let output_key = if outcome { "true" } else { "false" };
                            node_def
                                .next
                                .get(output_key)
                                .or_else(|| node_def.next.get("default"))
                                .ok_or_else(|| {
                                    Error::Validation(format!(
                                        "Condition node missing '{}' output path",
                                        output_key
                                    ))
                                })?
                        } else {
                            node_def
                                .next
                                .values()
                                .next()
                                .ok_or(Error::System("Logic node has no output".into()))?
                        };
                        session.current_node_id = next_id.clone();
                    }
                    StepType::Terminal => {
                        let terminal_span = info_span!(
                            "flow.node.terminal",
                            telemetry = "span",
                            node_id = %node_id,
                            step_type = %step_type
                        );
                        let _guard = terminal_span.enter();
                        let is_failure = node_def
                            .config
                            .get("is_failure")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        return if is_failure {
                            session.status = SessionStatus::Failed;
                            self.session_repo.update(&session).await?;
                            Ok(ExecutionResult::Failure {
                                reason: "Access Denied".into(),
                            })
                        } else {
                            session.status = SessionStatus::Completed;
                            self.session_repo.update(&session).await?;
                            Ok(ExecutionResult::Success {
                                redirect_url: "/".into(),
                            })
                        };
                    }
                    _ => return Err(Error::System("Unknown step type with no worker".into())),
                }
            }
        }
    }

    pub async fn resume_action(
        &self,
        realm_id: Uuid,
        token: &str,
    ) -> Result<(ExecutionResult, Uuid)> {
        let token_hash = hash_token(token);
        let action = self
            .action_repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(Error::InvalidActionToken)?;

        if action.realm_id != realm_id {
            return Err(Error::SecurityViolation(
                "Resume token does not belong to this realm".to_string(),
            ));
        }

        if action.is_expired() || action.is_consumed() {
            return Err(Error::InvalidActionToken);
        }

        self.action_repo.mark_consumed(&action.id).await?;

        let mut session = self
            .session_repo
            .find_by_id(&action.session_id)
            .await?
            .ok_or(Error::NotFound("Session not found".into()))?;

        if let Some(resume_node_id) = action.resume_node_id.clone() {
            session.current_node_id = resume_node_id;
        }

        session.status = SessionStatus::Active;
        clear_pending_action(&mut session);
        session.update_context(
            "action_result",
            serde_json::json!({
                "action_id": action.id.to_string(),
                "action_type": action.action_type,
            }),
        );
        session.update_context("action_payload", action.payload.clone());

        if action.action_type == "reset_credentials" {
            if let Some(audit_service) = &self.audit_service {
                let identifier_hash = action
                    .payload
                    .get("identifier")
                    .and_then(|value| value.as_str())
                    .map(hash_identifier);
                let metadata = serde_json::json!({
                    "action_type": action.action_type,
                    "identifier_hash": identifier_hash,
                });
                if let Err(err) = audit_service
                    .record(NewAuditEvent {
                        realm_id: session.realm_id,
                        actor_user_id: None,
                        action: "recovery.token_resumed".to_string(),
                        target_type: "auth_session_action".to_string(),
                        target_id: Some(action.id.to_string()),
                        metadata,
                    })
                    .await
                {
                    tracing::warn!("Failed to write recovery resume audit event: {}", err);
                }
            }
        }

        let session_id = session.id;
        self.session_repo.update(&session).await?;
        let result = self.execute(session_id, None).await?;
        Ok((result, session_id))
    }

    async fn heal_session(&self, session: &mut AuthenticationSession) -> Result<()> {
        let version = self
            .flow_store
            .get_version(&session.flow_version_id)
            .await?
            .ok_or(Error::System("Version not found".into()))?;

        let plan: ExecutionPlan = serde_json::from_str(&version.execution_artifact)
            .map_err(|e| Error::System(format!("Corrupt artifact: {}", e)))?;

        session.current_node_id = plan.start_node_id;
        session.status = SessionStatus::Active;
        session.user_id = None;
        clear_pending_action(session);

        self.session_repo.update(session).await?;
        Ok(())
    }
}

fn attach_template_key(mut context: Value, template_key: Option<&str>) -> Value {
    let Some(key) = template_key else {
        return context;
    };

    match context {
        Value::Object(ref mut map) => {
            map.insert("template_key".to_string(), Value::String(key.to_string()));
            context
        }
        other => serde_json::json!({
            "template_key": key,
            "payload": other,
        }),
    }
}

fn inject_node_config(session: &mut AuthenticationSession, config: &Value) -> Option<Value> {
    let previous = session.context.get("node_config").cloned();
    match session.context {
        Value::Object(ref mut map) => {
            map.insert("node_config".to_string(), config.clone());
        }
        _ => {
            session.context = serde_json::json!({ "node_config": config });
        }
    }
    previous
}

fn restore_node_config(session: &mut AuthenticationSession, previous: Option<Value>) {
    if let Value::Object(ref mut map) = session.context {
        if let Some(value) = previous {
            map.insert("node_config".to_string(), value);
        } else {
            map.remove("node_config");
        }
    }
}

fn is_condition_node(config: &Value) -> bool {
    config
        .get("logic_type")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value == "core.logic.condition")
}

fn evaluate_condition(context: &Value, config: &Value) -> Result<bool> {
    let path = config
        .get("context_path")
        .and_then(|value| value.as_str())
        .ok_or_else(|| Error::Validation("Condition node requires context_path".into()))?;
    let operator = config
        .get("operator")
        .and_then(|value| value.as_str())
        .unwrap_or("exists");

    let actual = resolve_context_path(context, path);
    let expected = parse_expected_value(config.get("compare_value"));
    if matches!(
        operator,
        "equals"
            | "not_equals"
            | "contains"
            | "starts_with"
            | "ends_with"
            | "gt"
            | "gte"
            | "lt"
            | "lte"
    ) && expected.is_none()
    {
        return Err(Error::Validation(format!(
            "Condition node requires compare_value for operator '{}'",
            operator
        )));
    }

    Ok(match operator {
        "exists" => actual.is_some_and(|value| !value.is_null()),
        "true" => actual.and_then(Value::as_bool).unwrap_or(false),
        "false" => actual.and_then(Value::as_bool).is_some_and(|value| !value),
        "equals" => compare_values(actual, expected.as_ref()).unwrap_or(false),
        "not_equals" => compare_values(actual, expected.as_ref())
            .map(|value| !value)
            .unwrap_or(false),
        "contains" => contains_value(actual, expected.as_ref()),
        "starts_with" => match (
            actual.and_then(Value::as_str),
            expected.as_ref().and_then(Value::as_str),
        ) {
            (Some(left), Some(right)) => left.starts_with(right),
            _ => false,
        },
        "ends_with" => match (
            actual.and_then(Value::as_str),
            expected.as_ref().and_then(Value::as_str),
        ) {
            (Some(left), Some(right)) => left.ends_with(right),
            _ => false,
        },
        "gt" => compare_numbers(actual, expected.as_ref(), |a, b| a > b),
        "gte" => compare_numbers(actual, expected.as_ref(), |a, b| a >= b),
        "lt" => compare_numbers(actual, expected.as_ref(), |a, b| a < b),
        "lte" => compare_numbers(actual, expected.as_ref(), |a, b| a <= b),
        _ => {
            return Err(Error::Validation(format!(
                "Unknown condition operator '{}'",
                operator
            )))
        }
    })
}

fn resolve_context_path<'a>(context: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = context;
    for segment in path.split('.') {
        if segment.is_empty() {
            continue;
        }
        match current {
            Value::Object(map) => {
                current = map.get(segment)?;
            }
            Value::Array(values) => {
                let index: usize = segment.parse().ok()?;
                current = values.get(index)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

fn parse_expected_value(value: Option<&Value>) -> Option<Value> {
    value.map(|raw| {
        if let Some(text) = raw.as_str() {
            serde_json::from_str::<Value>(text).unwrap_or_else(|_| Value::String(text.to_string()))
        } else {
            raw.clone()
        }
    })
}

fn compare_values(actual: Option<&Value>, expected: Option<&Value>) -> Option<bool> {
    let actual = actual?;
    let expected = expected?;
    Some(actual == expected)
}

fn contains_value(actual: Option<&Value>, expected: Option<&Value>) -> bool {
    match (actual, expected) {
        (Some(Value::String(left)), Some(Value::String(right))) => left.contains(right),
        (Some(Value::Array(values)), Some(expected_value)) => {
            values.iter().any(|value| value == expected_value)
        }
        _ => false,
    }
}

fn compare_numbers(
    actual: Option<&Value>,
    expected: Option<&Value>,
    cmp: impl Fn(f64, f64) -> bool,
) -> bool {
    let left = actual.and_then(as_number);
    let right = expected.and_then(as_number);
    match (left, right) {
        (Some(a), Some(b)) => cmp(a, b),
        _ => false,
    }
}

fn as_number(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|v| v as f64))
        .or_else(|| value.as_u64().map(|v| v as f64))
        .or_else(|| value.as_str().and_then(|v| v.parse::<f64>().ok()))
}

fn resolve_template_key(config: &Value) -> Option<String> {
    let explicit = config
        .get("template_key")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());
    if explicit.is_some() {
        return explicit;
    }

    let auth_type = config.get("auth_type").and_then(|value| value.as_str());
    match auth_type {
        Some("core.auth.password") => Some("login".to_string()),
        Some("core.auth.register") => Some("register".to_string()),
        Some("core.auth.forgot_credentials") => Some("forgot_credentials".to_string()),
        Some("core.auth.reset_password") => Some("reset_password".to_string()),
        Some("core.auth.verify_email_otp") => Some("verify_email".to_string()),
        Some("core.auth.otp") => Some("mfa".to_string()),
        Some("core.oidc.consent") => Some("consent".to_string()),
        _ => None,
    }
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(result)
}

fn hash_identifier(identifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(identifier.as_bytes());
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(result)
}

fn clear_pending_action(session: &mut AuthenticationSession) {
    if let serde_json::Value::Object(ref mut map) = session.context {
        map.remove("pending_action_id");
        map.remove("last_ui");
    }
}

fn extract_pending_ui(session: &AuthenticationSession) -> Option<(String, Value)> {
    let context = session.context.as_object()?;
    if !context.contains_key("pending_action_id") {
        return None;
    }
    let last_ui = context.get("last_ui")?.as_object()?;
    let screen_id = last_ui.get("screen_id")?.as_str()?.to_string();
    let ui_context = last_ui
        .get("context")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    Some((screen_id, ui_context))
}

struct AsyncSuspendRequest {
    action_type: String,
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    resume_node_id: Option<String>,
    payload: Value,
    screen: String,
    context: Value,
}

impl FlowExecutor {
    fn pending_action_result(
        &self,
        session: &AuthenticationSession,
        has_user_input: bool,
    ) -> Result<Option<ExecutionResult>> {
        if let Some((screen_id, context)) = extract_pending_ui(session) {
            if has_user_input {
                return Err(Error::Validation(
                    "Flow is waiting for an async action".into(),
                ));
            }
            return Ok(Some(ExecutionResult::AwaitingAction { screen_id, context }));
        }
        Ok(None)
    }

    async fn handle_async_suspend(
        &self,
        session: &mut AuthenticationSession,
        request: AsyncSuspendRequest,
    ) -> Result<ExecutionResult> {
        if let Some((screen_id, context)) = extract_pending_ui(session) {
            return Ok(ExecutionResult::AwaitingAction { screen_id, context });
        }

        let token_hash = hash_token(&request.token);
        let action_type = request.action_type.clone();
        let payload = request.payload.clone();
        let expires_at = request.expires_at;
        let email_expires_at = expires_at;
        let action = AuthSessionAction::new(
            session.id,
            session.realm_id,
            request.action_type,
            token_hash,
            request.payload,
            request.resume_node_id,
            expires_at,
        );

        self.action_repo.create(&action).await?;

        session.update_context(
            "pending_action_id",
            serde_json::json!(action.id.to_string()),
        );
        let mut ui_context = request.context.clone();

        let mut email_sent = false;
        if matches!(action_type.as_str(), "reset_credentials" | "email_verify") {
            match self
                .send_action_email(
                    &action_type,
                    &payload,
                    session.realm_id,
                    &request.token,
                    email_expires_at,
                    ui_context
                        .get("resume_path")
                        .and_then(|value| value.as_str()),
                )
                .await
            {
                Ok(true) => {
                    email_sent = true;
                    if let Some(ctx) = ui_context.as_object_mut() {
                        let message = if action_type == "reset_credentials" {
                            "If an account exists, a recovery email has been sent."
                        } else {
                            "If an account exists, a verification email has been sent."
                        };
                        ctx.insert("message".to_string(), serde_json::json!(message));
                        ctx.insert("delivery".to_string(), serde_json::json!("email"));
                    }
                }
                Ok(false) => {}
                Err(err) => {
                    tracing::warn!("Failed to send {} email: {}", action_type, err.to_string());
                }
            }
        }

        let screen_id = request.screen.clone();
        if action_type == "reset_credentials" {
            if let Some(audit_service) = &self.audit_service {
                let identifier_hash = payload
                    .get("identifier")
                    .and_then(|value| value.as_str())
                    .map(hash_identifier);
                let metadata = serde_json::json!({
                    "action_type": action_type,
                    "expires_at": action.expires_at,
                    "identifier_hash": identifier_hash,
                    "email_sent": email_sent,
                });
                if let Err(err) = audit_service
                    .record(NewAuditEvent {
                        realm_id: session.realm_id,
                        actor_user_id: None,
                        action: "recovery.token_issued".to_string(),
                        target_type: "auth_session_action".to_string(),
                        target_id: Some(action.id.to_string()),
                        metadata,
                    })
                    .await
                {
                    tracing::warn!("Failed to write recovery audit event: {}", err);
                }
            }
        }
        let response_context = ui_context.clone();
        session.update_context(
            "last_ui",
            serde_json::json!({
                "screen_id": screen_id,
                "context": ui_context,
                "updated_at": Utc::now(),
            }),
        );

        self.session_repo.update(session).await?;

        Ok(ExecutionResult::AwaitingAction {
            screen_id: request.screen,
            context: response_context,
        })
    }

    async fn send_action_email(
        &self,
        action_type: &str,
        payload: &Value,
        realm_id: Uuid,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
        resume_path_fallback: Option<&str>,
    ) -> Result<bool> {
        let Some(service) = &self.email_delivery else {
            return Ok(false);
        };

        let identifier = payload
            .get("identifier")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        if identifier.trim().is_empty() {
            return Ok(false);
        }

        if action_type == "reset_credentials" {
            let user_id = payload
                .get("user_id")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if user_id.trim().is_empty() {
                return Ok(false);
            }
        }

        let resume_path = payload
            .get("resume_path")
            .and_then(|value| value.as_str())
            .or(resume_path_fallback)
            .unwrap_or_else(|| {
                if action_type == "reset_credentials" {
                    "/forgot-password"
                } else {
                    "/register"
                }
            });

        match action_type {
            "reset_credentials" => {
                service
                    .send_recovery_email(
                        &realm_id,
                        RecoveryEmail {
                            identifier: identifier.to_string(),
                            token: token.to_string(),
                            expires_at,
                            resume_path: resume_path.to_string(),
                        },
                    )
                    .await
            }
            "email_verify" => {
                let subject = payload
                    .get("email_subject")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string());
                let body = payload
                    .get("email_body")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string());
                service
                    .send_verification_email(
                        &realm_id,
                        VerificationEmail {
                            identifier: identifier.to_string(),
                            token: token.to_string(),
                            expires_at,
                            resume_path: resume_path.to_string(),
                            subject,
                            body,
                        },
                    )
                    .await
            }
            _ => Ok(false),
        }
    }

    pub async fn resend_action(&self, realm_id: Uuid, token: &str) -> Result<bool> {
        let token_hash = hash_token(token);
        let action = self
            .action_repo
            .find_by_token_hash(&token_hash)
            .await?
            .ok_or(Error::InvalidActionToken)?;

        if action.realm_id != realm_id {
            return Err(Error::SecurityViolation(
                "Resume token does not belong to this realm".to_string(),
            ));
        }

        if action.is_expired() || action.is_consumed() {
            return Err(Error::InvalidActionToken);
        }

        self.send_action_email(
            &action.action_type,
            &action.payload,
            action.realm_id,
            token,
            action.expires_at,
            None,
        )
        .await
    }
}

#[cfg(test)]
mod tests;
