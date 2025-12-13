use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::auth_flow::{AuthContext, AuthStepResult, LoginSession};
use crate::domain::auth_session::SessionStatus;
use crate::domain::execution::{ExecutionPlan, ExecutionResult, StepType};
use crate::error::{Error, Result};
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::flow_store::FlowStore;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct FlowExecutor {
    session_repo: Arc<dyn AuthSessionRepository>,
    flow_store: Arc<dyn FlowStore>,
    registry: Arc<RuntimeRegistry>,
}

impl FlowExecutor {
    pub fn new(
        session_repo: Arc<dyn AuthSessionRepository>,
        flow_store: Arc<dyn FlowStore>,
        registry: Arc<RuntimeRegistry>,
    ) -> Self {
        Self {
            session_repo,
            flow_store,
            registry,
        }
    }

    /// Main Execution Loop
    pub async fn execute(
        &self,
        session_id: Uuid,
        mut user_input: Option<Value>, // Mutable to allow discarding input on reset
    ) -> Result<ExecutionResult> {
        // 1. Load Session
        let mut session = self
            .session_repo
            .find_by_id(&session_id)
            .await?
            .ok_or(Error::NotFound("Session not found".to_string()))?;

        // [FIX START] SELF-HEALING: Check Status FIRST.
        // If the session is closed/completed, we must RESET it to the beginning.
        // We cannot process input for a closed session, so we discard the input.
        if session.status != SessionStatus::active {
            // A. We need the Flow ID to find the active version.
            // We get it from the *current* version associated with the session.
            let old_version = self
                .flow_store
                .get_version(&session.flow_version_id)
                .await?
                .ok_or(Error::System(
                    "Current flow version definition missing".to_string(),
                ))?;

            // FIX: Parse the String ID to a Uuid before passing it to the store
            // We handle the possibility that the ID in the DB might be invalid
            let flow_id_uuid = Uuid::parse_str(&old_version.flow_id.to_string()).map_err(|_| {
                Error::System("Invalid flow_id format in version definition".to_string())
            })?;

            // B. Fetch the CURRENT ACTIVE version
            let active_version = self
                .flow_store
                .get_active_version(&flow_id_uuid) // Now passing &Uuid
                .await?
                .ok_or(Error::System(
                    "No active version found for this flow".to_string(),
                ))?;

            println!(
                "INFO: Resetting Session {}. Upgrading from v{} to v{}",
                session_id,
                old_version.version_number,
                active_version.version_number // Assuming .version_number exists
            );

            // C. Parse the NEW Plan from the active version
            let plan: ExecutionPlan = serde_json::from_str(&active_version.execution_artifact)
                .map_err(|e| {
                    Error::Unexpected(anyhow::anyhow!("Corrupt artifact in active version: {}", e))
                })?;

            // D. Upgrade and Reset the Session
            session.flow_version_id = Uuid::parse_str(&active_version.id.to_string())?;
            session.current_node_id = plan.start_node_id; // <--- Point to start of new graph
            session.status = SessionStatus::active; // <--- Re-open session
            session.user_id = None; // <--- Clear old user identity

            // IMPORTANT: We do NOT clear `session.context`.
            // If this was an OIDC Re-auth flow, the 'redirect_uri' and 'nonce' are in context.
            // We want to keep those so the user is eventually redirected back to the app correctly.

            // E. Persist Changes
            self.session_repo.update(&session).await?;

            // F. Discard Input
            // The user input (e.g. "Login") was targeted at the old, closed session state.
            // We discard it so the executor simply returns the "Start Screen" of the new version.
            user_input = None;
        }
        // [FIX END] ---------------------------------------------------------------

        // 2. Load the Graph (ExecutionPlan)
        // We fetch the immutable version tied to this session
        let version = self
            .flow_store
            .get_version(&session.flow_version_id)
            .await?;

        let version = version.ok_or(Error::System("Flow version missing".to_string()))?;

        let plan: ExecutionPlan = serde_json::from_str(&version.execution_artifact)
            .map_err(|e| Error::Unexpected(anyhow::anyhow!("Corrupt artifact: {}", e)))?;

        // 3. Handle Input (Resume from a UI State)
        if let Some(input) = user_input {
            let current_node =
                plan.nodes
                    .get(&session.current_node_id)
                    .ok_or(Error::Validation(
                        "Current node not found in plan".to_string(),
                    ))?;

            if current_node.step_type == StepType::Authenticator {
                // A. Find the Implementation (The Worker)
                let auth_key = current_node
                    .config
                    .get("auth_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("core.auth.password");

                let authenticator =
                    self.registry
                        .get_authenticator(auth_key)
                        .ok_or(Error::System(format!(
                            "No authenticator found for type: {}",
                            auth_key
                        )))?;

                // B. Prepare Context
                let credentials = self.parse_credentials(&input);

                // Map to legacy LoginSession structure for the authenticator trait
                let legacy_login_session = LoginSession {
                    id: session.id,
                    realm_id: session.realm_id,
                    flow_id: Uuid::default(),
                    current_step: 0,
                    user_id: session.user_id,
                    state_data: None,
                    context: session.context.clone(), // Pass context (OIDC data) through
                    expires_at: session.expires_at,
                };

                let mut context = AuthContext {
                    realm_id: session.realm_id,
                    credentials,
                    login_session: legacy_login_session,
                    config: Default::default(),
                };

                // C. Execute Logic
                let result = authenticator.execute(&mut context).await?;

                // D. Determine Outcome Edge
                let edge_label = match result {
                    AuthStepResult::Success => {
                        // Retrieve the ID from the worker's context
                        if let Some(uid) = context.login_session.user_id {
                            session.user_id = Some(uid);
                            // IMPORTANT: Save to DB immediately to persist user_id
                            self.session_repo.update(&session).await?;
                        } else {
                            return Err(Error::System(
                                "Authenticator succeeded but returned no User ID".to_string(),
                            ));
                        }
                        "success"
                    }
                    AuthStepResult::Failure { .. } => "failure",
                    _ => "default",
                };

                // E. Move Pointer
                if let Some(next_id) = current_node.next.get(edge_label) {
                    session.current_node_id = next_id.clone();
                } else {
                    // If we failed and there is no "failure" wire, return error to UI
                    if let AuthStepResult::Failure { message } = result {
                        self.session_repo.update(&session).await?;
                        return Ok(ExecutionResult::Failure { reason: message });
                    }
                    return Err(Error::Validation(
                        "Flow stuck: No path for this outcome".to_string(),
                    ));
                }
            } else {
                // This error block will now only hit if a user manually POSTs to a logic node,
                // because the "closed session" case is handled at the top.
                return Err(Error::Validation(
                    "Input received for non-interactive step".to_string(),
                ));
            }
        }

        // 4. Execution Loop (Traverse the Graph until we hit a UI screen or End)
        loop {
            // Persist state at every tick
            self.session_repo.update(&session).await?;

            let node = plan
                .nodes
                .get(&session.current_node_id)
                .ok_or(Error::Validation("Node missing from graph".to_string()))?;

            match node.step_type {
                // STOP: UI Screen
                StepType::Authenticator => {
                    let auth_key = node
                        .config
                        .get("auth_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("core.auth.password");

                    let authenticator =
                        self.registry
                            .get_authenticator(auth_key)
                            .ok_or(Error::System(format!(
                                "Authenticator {} not found",
                                auth_key
                            )))?;

                    // Prepare context for challenge
                    let context = AuthContext {
                        realm_id: session.realm_id,
                        credentials: HashMap::new(),
                        // Populate LoginSession correctly for context access
                        login_session: LoginSession {
                            id: session.id,
                            realm_id: session.realm_id,
                            flow_id: Uuid::default(),
                            current_step: 0,
                            user_id: session.user_id,
                            state_data: None,
                            context: session.context.clone(),
                            expires_at: session.expires_at,
                        },
                        config: Default::default(),
                    };

                    let challenge_result = authenticator.challenge(&context).await?;

                    return if let AuthStepResult::Challenge { challenge_name, .. } =
                        challenge_result
                    {
                        // CRITICAL: Save state before returning to UI
                        // This ensures 'current_node_id' is saved so refresh works.
                        session.context = context.login_session.context;
                        self.session_repo.update(&session).await?;

                        Ok(ExecutionResult::Challenge {
                            screen_id: challenge_name,
                            context: session.context.clone(),
                        })
                    } else {
                        Err(Error::System(
                            "Authenticator did not return a challenge".to_string(),
                        ))
                    };
                }

                // RUN: Logic Node
                StepType::Logic => {
                    let next_node_id = node
                        .next
                        .get("true")
                        .or_else(|| node.next.get("default"))
                        .or_else(|| node.next.get("next"))
                        .or_else(|| {
                            // Fallback: If only 1 path exists, take it.
                            if node.next.len() == 1 {
                                node.next.values().next()
                            } else {
                                None
                            }
                        });

                    if let Some(next_id) = next_node_id {
                        session.current_node_id = next_id.clone();
                        // Loop continues immediately to the next node
                    } else {
                        return Err(Error::Validation(format!(
                            "Logic node '{}' stuck. Available edges: {:?}",
                            node.id,
                            node.next.keys()
                        )));
                    }
                }

                // END: Terminal Node
                StepType::Terminal => {
                    let is_failure = node
                        .config
                        .get("is_failure")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if is_failure {
                        session.status = SessionStatus::failed;
                        self.session_repo.update(&session).await?;
                        return Ok(ExecutionResult::Failure {
                            reason: "Access Denied".to_string(),
                        });
                    } else {
                        session.status = SessionStatus::completed;
                        self.session_repo.update(&session).await?;

                        // Calculate final redirect (OIDC Callback or Default)
                        return Ok(ExecutionResult::Success {
                            redirect_url: "/".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Helper to convert JSON input to the HashMap your Authenticator expects
    fn parse_credentials(&self, input: &Value) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(obj) = input.as_object() {
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    map.insert(k.clone(), s.to_string());
                }
            }
        }
        map
    }
}
