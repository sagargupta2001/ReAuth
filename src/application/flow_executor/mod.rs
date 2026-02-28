use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::runtime_registry::RuntimeRegistry;
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
}

impl FlowExecutor {
    pub fn new(
        session_repo: Arc<dyn AuthSessionRepository>,
        flow_store: Arc<dyn FlowStore>,
        registry: Arc<RuntimeRegistry>,
        action_repo: Arc<dyn AuthSessionActionRepository>,
    ) -> Self {
        Self {
            session_repo,
            flow_store,
            registry,
            action_repo,
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

                match worker
                    .handle_input(&mut session, input)
                    .instrument(handle_span)
                    .await?
                {
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
                        session.current_node_id = next_id.clone();
                    }
                    NodeOutcome::Reject { .. } => {
                        self.session_repo.update(&session).await?;

                        let exec_span = info_span!(
                            "flow.node.execute",
                            telemetry = "span",
                            node_id = %node_id,
                            step_type = %step_type,
                            worker = %worker_key
                        );
                        let ui_outcome = worker.execute(&mut session).instrument(exec_span).await?;
                        if let NodeOutcome::SuspendForUI { screen, context } = ui_outcome {
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
                    _ => return Err(Error::System("Unexpected outcome from handle_input".into())),
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

            let worker = if node_def.step_type == StepType::Authenticator {
                let key = node_def
                    .config
                    .get("auth_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("core.auth.password");
                self.registry.get_node(key)
            } else {
                None
            };

            if let Some(worker) = worker {
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
                        session.current_node_id = next_id.clone();
                    }
                    NodeOutcome::SuspendForUI { screen, context } => {
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
                        self.session_repo.update(&session).await?;
                        return Ok(ExecutionResult::Success {
                            redirect_url: "/".to_string(),
                        });
                    }
                    NodeOutcome::FlowFailure { reason } => {
                        session.status = SessionStatus::Failed;
                        self.session_repo.update(&session).await?;
                        return Ok(ExecutionResult::Failure { reason });
                    }
                    _ => return Err(Error::System("Unhandled execution outcome".into())),
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
                        let next_id = node_def
                            .next
                            .values()
                            .next()
                            .ok_or(Error::System("Logic node has no output".into()))?;
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

    pub async fn resume_action(&self, realm_id: Uuid, token: &str) -> Result<ExecutionResult> {
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

        self.session_repo.update(&session).await?;
        self.execute(session.id, None).await
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

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
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
        let action = AuthSessionAction::new(
            session.id,
            session.realm_id,
            request.action_type,
            token_hash,
            request.payload,
            request.resume_node_id,
            request.expires_at,
        );

        self.action_repo.create(&action).await?;

        session.update_context(
            "pending_action_id",
            serde_json::json!(action.id.to_string()),
        );
        let screen_id = request.screen.clone();
        let ui_context = request.context.clone();
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
            context: request.context,
        })
    }
}

#[cfg(test)]
mod tests;
