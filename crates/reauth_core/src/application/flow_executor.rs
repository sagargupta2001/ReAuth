use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::auth_flow::{AuthContext, AuthStepResult};
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
        user_input: Option<Value>,
    ) -> Result<ExecutionResult> {
        // 1. Load Session
        let mut session = self
            .session_repo
            .find_by_id(&session_id)
            .await?
            .ok_or(Error::NotFound("Session not found".to_string()))?;

        if session.status != SessionStatus::Active {
            return Err(Error::Validation("Session is closed".to_string()));
        }

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
                // We use the node's config "auth_type" OR fall back to the node ID if simple
                let auth_key = current_node
                    .config
                    .get("auth_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("core.auth.password"); // Default for now, ideally strictly typed

                let authenticator =
                    self.registry
                        .get_authenticator(auth_key)
                        .ok_or(Error::System(format!(
                            "No authenticator found for type: {}",
                            auth_key
                        )))?;

                // B. Prepare Context
                // Map the JSON input to the HashMap credentials your trait expects
                let credentials = self.parse_credentials(&input);

                // Note: We are creating a temporary AuthContext.
                // Ideally, you should update your Authenticator trait to accept `AuthenticationSession` directly.
                let mut context = AuthContext {
                    realm_id: session.realm_id,
                    credentials,
                    // TODO: You might need to bridge your new AuthenticationSession to the old LoginSession struct
                    // or refactor the Authenticator trait to use the new struct.
                    // For this snippet, we assume the trait is updated or we pass minimal data.
                    login_session: Default::default(), // Placeholder if you haven't refactored trait yet
                    config: Default::default(),
                };

                // C. Execute Logic
                let result = authenticator.execute(&mut context).await?;

                // D. Determine Outcome Edge
                let edge_label = match result {
                    AuthStepResult::Success => {
                        // --- FIX STARTS HERE ---
                        // 1. Retrieve the ID from the worker's context
                        if let Some(uid) = context.login_session.user_id {
                            session.user_id = Some(uid);

                            // 2. IMPORTANT: Save to DB immediately
                            self.session_repo.update(&session).await?;
                        } else {
                            // If Success returned but no ID, that's a logic bug in the Authenticator
                            return Err(Error::System(
                                "Authenticator succeeded but returned no User ID".to_string(),
                            ));
                        }
                        // --- FIX ENDS HERE ---

                        "success"
                    }
                    AuthStepResult::Failure { .. } => "failure",
                    _ => "default",
                };

                // E. Move Pointer
                if let Some(next_id) = current_node.next.get(edge_label) {
                    session.current_node_id = next_id.clone();
                } else {
                    // If we failed and there is no "failure" wire, we return the error to UI
                    if let AuthStepResult::Failure { message } = result {
                        self.session_repo.update(&session).await?;
                        return Ok(ExecutionResult::Failure { reason: message });
                    }
                    return Err(Error::Validation(
                        "Flow stuck: No path for this outcome".to_string(),
                    ));
                }
            } else {
                return Err(Error::Validation(
                    "Input received for non-interactive step".to_string(),
                ));
            }
        }

        // 4. Execution Loop (Traverse the Graph)
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
                    // We need to ask the Authenticator *what* screen to show
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

                    // Prepare empty context for challenge
                    let context = AuthContext {
                        realm_id: session.realm_id,
                        credentials: HashMap::new(),
                        login_session: Default::default(),
                        config: Default::default(),
                    };

                    let challenge_result = authenticator.challenge(&context).await?;

                    return if let AuthStepResult::Challenge { challenge_name, .. } =
                        challenge_result
                    {
                        // CRITICAL: We calculated the next state/screen.
                        // We MUST save it to the DB now. Otherwise, a page refresh
                        // reloads the *previous* state from the DB.

                        // Update context/pointer in the session object
                        session.context = context.login_session.context;
                        self.session_repo.update(&session).await?;
                        Ok(ExecutionResult::Challenge {
                            screen_id: challenge_name, // Should return "username_password"
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
                    // FIX: Don't just look for "true".
                    // The Start node usually maps to "default" or "next".
                    // A Condition node maps to "true"/"false".

                    let next_node_id = node
                        .next
                        .get("true") // 1. Try Boolean True
                        .or_else(|| node.next.get("default")) // 2. Try Generic/Default (Common for Start)
                        .or_else(|| node.next.get("next")) // 3. Try Named 'Next'
                        .or_else(|| {
                            // 4. Fallback: If only 1 path exists, take it.
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
                        // Helpful error message to debug exactly which node and keys are failing
                        return Err(Error::Validation(format!(
                            "Logic node '{}' (type: {}) has no matching output path. Available edges: {:?}",
                            node.id,
                            node.config.get("type").and_then(|v| v.as_str()).unwrap_or("unknown"),
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
                        session.status = SessionStatus::Failed;
                        self.session_repo.update(&session).await?;
                        return Ok(ExecutionResult::Failure {
                            reason: "Access Denied".to_string(),
                        });
                    } else {
                        session.status = SessionStatus::Completed;
                        self.session_repo.update(&session).await?;

                        // Issue Tokens here if needed, or redirect to callback
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
