use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::runtime_registry::RuntimeRegistry;
use crate::domain::auth_session::SessionStatus;
use crate::domain::execution::lifecycle::NodeOutcome;
use crate::domain::execution::{ExecutionPlan, StepType};
use crate::error::{Error, Result};
use crate::ports::auth_session_repository::AuthSessionRepository;
use crate::ports::flow_store::FlowStore;

/// The result returned to the API Handler.
#[derive(Debug)]
pub enum EngineResult {
    /// Render a UI Screen (e.g. Login Form)
    ShowUI { screen_id: String, context: Value },
    /// Redirect the user (Flow Complete)
    Redirect { url: String },
    /// Stop and wait (e.g. Email Sent)
    Wait,
}

pub struct FlowEngine {
    registry: Arc<RuntimeRegistry>,
    flow_store: Arc<dyn FlowStore>,
    session_repo: Arc<dyn AuthSessionRepository>,
}

impl FlowEngine {
    pub fn new(
        registry: Arc<RuntimeRegistry>,
        flow_store: Arc<dyn FlowStore>,
        session_repo: Arc<dyn AuthSessionRepository>,
    ) -> Self {
        Self {
            registry,
            flow_store,
            session_repo,
        }
    }

    /// The Main Entry Point.
    /// Called by POST /execute
    pub async fn execute(
        &self,
        session_id: Uuid,
        user_input: Option<Value>,
    ) -> Result<EngineResult> {
        // 1. Load Session
        let mut session = self
            .session_repo
            .find_by_id(&session_id)
            .await?
            .ok_or(Error::NotFound("Session not found".into()))?;

        if session.status != SessionStatus::Active {
            return Err(Error::Validation("Session is not active".into()));
        }

        // 2. Load Graph
        let version = self
            .flow_store
            .get_version(&session.flow_version_id)
            .await?
            .ok_or(Error::System("Flow version missing".into()))?;

        let plan: ExecutionPlan = serde_json::from_str(&version.execution_artifact)
            .map_err(|e| Error::System(format!("Corrupt artifact: {}", e)))?;

        // 3. Handle Input (If this is a POST from a UI)
        if let Some(input) = user_input {
            let current_node_def = plan
                .nodes
                .get(&session.current_node_id)
                .ok_or(Error::System("Current node not found in plan".into()))?;

            // We only process input if the node is an Authenticator
            if current_node_def.step_type == StepType::Authenticator {
                // Find the worker
                let worker_key = current_node_def
                    .config
                    .get("auth_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("core.auth.password"); // Fallback or read from node type mapping

                let worker = self
                    .registry
                    .get_node(worker_key)
                    .ok_or(Error::System(format!("Worker not found: {}", worker_key)))?;

                // [LIFECYCLE PHASE 3]: Handle Input
                let outcome = worker.handle_input(&mut session, input).await?;

                match outcome {
                    NodeOutcome::Continue { output } => {
                        // Success! Transition edge.
                        let next_id = current_node_def
                            .next
                            .get(&output)
                            .or_else(|| current_node_def.next.get("default"))
                            .ok_or(Error::Validation("No path forward from this input".into()))?;

                        // [LIFECYCLE PHASE 4]: On Exit (Old Node)
                        worker.on_exit(&mut session).await?;

                        session.current_node_id = next_id.clone();
                        // Proceed to main loop
                    }
                    NodeOutcome::Reject { error: _ } => {
                        // Failure! Stay here and show error.
                        self.session_repo.update(&session).await?;
                        // Re-run execute to get the UI again (with error context)
                        let ui = worker.execute(&mut session).await?;
                        if let NodeOutcome::SuspendForUI { screen, context } = ui {
                            return Ok(EngineResult::ShowUI {
                                screen_id: screen,
                                context,
                            });
                        }
                        return Err(Error::System("Rejecting node did not return UI".into()));
                    }
                    _ => return Err(Error::System("Unexpected outcome from handle_input".into())),
                }
            }
        }

        // 4. Main State Machine Loop
        // Keep advancing until we hit a UI, Async Wait, or Terminal.
        loop {
            let node_def = plan
                .nodes
                .get(&session.current_node_id)
                .ok_or(Error::System("Node missing from graph".into()))?;

            // Resolve Worker (default to logic handlers if needed, or pass-through)
            // For now, assuming "core.auth.password" is both the type and the worker key
            // You might need a mapping here if node.type != worker_key
            let worker_key = match node_def.step_type {
                StepType::Authenticator => node_def
                    .config
                    .get("auth_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("core.auth.password"),
                // For Logic/Terminal, we can have generic workers or specific ones
                _ => node_def
                    .config
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("core.start"), // Simplified
            };

            // If it's a pure logic/terminal node that has no worker (e.g. Start), manually handle or register a "NoOp" worker.
            // For this implementation, let's assume ALL nodes have a registered worker (even "core.start").

            if let Some(worker) = self.registry.get_node(worker_key) {
                // [LIFECYCLE PHASE 1]: On Enter
                worker.on_enter(&mut session).await?;

                // [LIFECYCLE PHASE 2]: Execute
                let outcome = worker.execute(&mut session).await?;

                match outcome {
                    NodeOutcome::Continue { output } => {
                        let next_id = node_def
                            .next
                            .get(&output)
                            .or_else(|| node_def.next.get("default"))
                            .ok_or(Error::System(format!("No edge for output '{}'", output)))?;

                        worker.on_exit(&mut session).await?;
                        session.current_node_id = next_id.clone();
                        // Loop continues...
                    }
                    NodeOutcome::SuspendForUI { screen, context } => {
                        // Stop! Persist state and return UI.
                        self.session_repo.update(&session).await?;
                        return Ok(EngineResult::ShowUI {
                            screen_id: screen,
                            context,
                        });
                    }
                    NodeOutcome::FlowSuccess { .. } => {
                        session.status = SessionStatus::Completed;
                        self.session_repo.update(&session).await?;
                        return Ok(EngineResult::Redirect {
                            url: "/".to_string(),
                        }); // TODO: OIDC Callback
                    }
                    NodeOutcome::FlowFailure { reason } => {
                        session.status = SessionStatus::Failed;
                        self.session_repo.update(&session).await?;
                        // Return error page
                        return Ok(EngineResult::ShowUI {
                            screen_id: "error".to_string(),
                            context: serde_json::json!({ "message": reason }),
                        });
                    }
                    _ => return Err(Error::System("Unhandled execution outcome".into())),
                }
            } else {
                // Handle Generic Logic/Terminal if no worker exists (e.g., Start Node)
                if node_def.step_type == StepType::Logic {
                    // Simple pass-through for Start Node
                    let next_id = node_def
                        .next
                        .values()
                        .next()
                        .ok_or(Error::System("Logic node has no output".into()))?;
                    session.current_node_id = next_id.clone();
                } else {
                    return Err(Error::System(format!("No worker for node {}", worker_key)));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
