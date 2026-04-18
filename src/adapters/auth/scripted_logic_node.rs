use crate::application::script_engine::{
    execute_with_limits, ScriptEngine, ScriptExecutionContext, ScriptExecutionLimits,
};
use crate::domain::auth_session::AuthenticationSession;
use crate::domain::execution::lifecycle::{LifecycleNode, NodeOutcome};
use crate::error::{Error, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::sync::Arc;
use tracing::instrument;

pub struct ScriptedLogicNode {
    engine: Arc<dyn ScriptEngine>,
}

impl ScriptedLogicNode {
    pub fn new(engine: Arc<dyn ScriptEngine>) -> Self {
        Self { engine }
    }
}

#[derive(Debug, Deserialize)]
struct ScriptedLogicConfig {
    script: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ScriptedLogicResult {
    #[serde(default)]
    output: Option<String>,
    #[serde(default)]
    context: Option<Value>,
    #[serde(default)]
    remove_keys: Option<Vec<String>>,
}

fn load_config(session: &AuthenticationSession) -> Result<ScriptedLogicConfig> {
    let config = session
        .context
        .get("node_config")
        .cloned()
        .unwrap_or_else(|| json!({}));
    serde_json::from_value(config)
        .map_err(|err| Error::Validation(format!("Invalid scripted logic config: {}", err)))
}

fn merge_context(session: &mut AuthenticationSession, updates: Map<String, Value>) {
    match session.context {
        Value::Object(ref mut map) => {
            map.extend(updates);
        }
        _ => {
            session.context = Value::Object(updates);
        }
    }
}

fn apply_result(session: &mut AuthenticationSession, result: Value) -> Result<NodeOutcome> {
    let parsed: ScriptedLogicResult = serde_json::from_value(result).map_err(|err| {
        Error::Validation(format!(
            "Scripted logic result must be an object with valid fields: {}",
            err
        ))
    })?;

    if let Some(remove_keys) = parsed.remove_keys {
        if let Value::Object(ref mut map) = session.context {
            for key in remove_keys {
                let trimmed = key.trim();
                if !trimmed.is_empty() {
                    map.remove(trimmed);
                }
            }
        }
    }

    if let Some(context) = parsed.context {
        let Value::Object(updates) = context else {
            return Err(Error::Validation(
                "Scripted logic result.context must be an object".to_string(),
            ));
        };
        merge_context(session, updates);
    }

    let output = parsed
        .output
        .unwrap_or_else(|| "success".to_string())
        .trim()
        .to_string();

    match output.as_str() {
        "success" | "failure" => Ok(NodeOutcome::Continue { output }),
        _ => Err(Error::Validation(format!(
            "Unsupported scripted logic output '{}'",
            output
        ))),
    }
}

#[async_trait]
impl LifecycleNode for ScriptedLogicNode {
    #[instrument(
        skip_all,
        fields(telemetry = "span", node = "scripted_logic", phase = "execute")
    )]
    async fn execute(&self, session: &mut AuthenticationSession) -> Result<NodeOutcome> {
        let config = load_config(session)?;
        let script = config
            .script
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| Error::Validation("Scripted logic node missing script".to_string()))?;

        let signal = session
            .context
            .get("signal")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let result = execute_with_limits(
            self.engine.clone(),
            script,
            ScriptExecutionContext {
                input: json!({}),
                context: session.context.clone(),
                signal,
            },
            ScriptExecutionLimits::for_logic(),
        )
        .await?;

        apply_result(session, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::script_engine::BoaScriptEngine;
    use serde_json::json;
    use uuid::Uuid;

    fn build_session(script: &str) -> AuthenticationSession {
        let mut session = AuthenticationSession::new(Uuid::new_v4(), Uuid::new_v4(), "node".into());
        session.context = json!({
            "node_config": {
                "script": script
            },
            "existing": true
        });
        session
    }

    #[tokio::test]
    async fn scripted_logic_returns_continue_and_merges_context() {
        let node = ScriptedLogicNode::new(Arc::new(BoaScriptEngine));
        let mut session = build_session(
            "return { output: 'success', context: { risk_score: 42, existing: false } };",
        );

        let outcome = node.execute(&mut session).await.expect("execute");

        match outcome {
            NodeOutcome::Continue { output } => assert_eq!(output, "success"),
            other => panic!("unexpected outcome: {other:?}"),
        }
        assert_eq!(session.context.get("risk_score"), Some(&json!(42)));
        assert_eq!(session.context.get("existing"), Some(&json!(false)));
    }

    #[tokio::test]
    async fn scripted_logic_can_remove_keys() {
        let node = ScriptedLogicNode::new(Arc::new(BoaScriptEngine));
        let mut session = build_session("return { remove_keys: ['existing'] };");

        let outcome = node.execute(&mut session).await.expect("execute");

        match outcome {
            NodeOutcome::Continue { output } => assert_eq!(output, "success"),
            other => panic!("unexpected outcome: {other:?}"),
        }
        assert!(session.context.get("existing").is_none());
    }
}
