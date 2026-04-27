use crate::application::script_engine::{
    execute_with_limits, ScriptEngine, ScriptExecutionContext, ScriptExecutionLimits,
};
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::NodeOutcome;
use crate::error::{Error, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::instrument;

pub struct ScriptedUiAuthenticator {
    engine: Arc<dyn ScriptEngine>,
}

impl ScriptedUiAuthenticator {
    pub fn new(engine: Arc<dyn ScriptEngine>) -> Self {
        Self { engine }
    }
}

#[derive(Debug, Deserialize)]
struct ScriptedUiConfig {
    script: Option<String>,
    screen_id: Option<String>,
    ui_context: Option<Value>,
    signal_handlers: Option<HashMap<String, String>>,
}

fn load_config(session: &AuthenticationSession) -> Result<ScriptedUiConfig> {
    let config = session
        .context
        .get("node_config")
        .cloned()
        .unwrap_or_else(|| json!({}));
    serde_json::from_value(config)
        .map_err(|err| Error::Validation(format!("Invalid scripted UI config: {}", err)))
}

fn resolve_screen_id(config: &ScriptedUiConfig) -> String {
    config
        .screen_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "core.ui.scripted".to_string())
}

fn resolve_script(config: &ScriptedUiConfig, signal_type: Option<&str>) -> Option<String> {
    if let Some(signal_type) = signal_type {
        if let Some(map) = &config.signal_handlers {
            if let Some(script) = map.get(signal_type) {
                if !script.trim().is_empty() {
                    return Some(script.clone());
                }
            }
        }
    }
    config
        .script
        .clone()
        .filter(|script| !script.trim().is_empty())
}

fn outcome_from_script(result: Value, default_screen: &str) -> Result<NodeOutcome> {
    let Value::Object(map) = result else {
        return Err(Error::Validation(
            "Scripted UI handler must return an object".to_string(),
        ));
    };

    let outcome = map
        .get("outcome")
        .and_then(Value::as_str)
        .unwrap_or("continue");

    match outcome {
        "continue" => {
            let output = map
                .get("output")
                .and_then(Value::as_str)
                .unwrap_or("success");
            Ok(NodeOutcome::Continue {
                output: output.to_string(),
            })
        }
        "success" => Ok(NodeOutcome::Continue {
            output: "success".to_string(),
        }),
        "reject" => {
            let error = map
                .get("error")
                .or_else(|| map.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("Rejected by script");
            Ok(NodeOutcome::Reject {
                error: error.to_string(),
            })
        }
        "failure" => {
            let reason = map
                .get("reason")
                .or_else(|| map.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("Scripted node failed");
            Ok(NodeOutcome::FlowFailure {
                reason: reason.to_string(),
            })
        }
        "challenge" => {
            let screen = map
                .get("screen")
                .and_then(Value::as_str)
                .unwrap_or(default_screen);
            let context = map.get("context").cloned().unwrap_or_else(|| json!({}));
            Ok(NodeOutcome::SuspendForUI {
                screen: screen.to_string(),
                context,
            })
        }
        _ => Err(Error::Validation(format!(
            "Unsupported scripted outcome '{}'",
            outcome
        ))),
    }
}

#[async_trait]
impl crate::domain::execution::lifecycle::LifecycleNode for ScriptedUiAuthenticator {
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "scripted_ui", phase = "execute")
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let config = load_config(session)?;
        let screen = resolve_screen_id(&config);
        let context = config.ui_context.unwrap_or_else(|| json!({}));
        Ok(NodeOutcome::SuspendForUI { screen, context })
    }

    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "scripted_ui", phase = "handle_input")
    )]
    async fn handle_input(
        &self,
        session: &mut AuthenticationSession,
        input: Value,
    ) -> Result<NodeOutcome> {
        let config = load_config(session)?;
        let signal = session
            .context
            .get("signal")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let signal_type = signal.get("type").and_then(Value::as_str);
        let script = resolve_script(&config, signal_type)
            .ok_or_else(|| Error::Validation("Scripted UI node missing script".to_string()))?;

        let result = execute_with_limits(
            self.engine.clone(),
            script,
            ScriptExecutionContext {
                input,
                context: session.context.clone(),
                signal,
            },
            ScriptExecutionLimits::for_ui(),
        )
        .await?;

        let screen = resolve_screen_id(&config);
        outcome_from_script(result, &screen)
    }
}
