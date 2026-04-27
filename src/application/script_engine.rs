use boa_engine::{Context, Source};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::error::{Error, Result};

pub struct ScriptExecutionContext {
    pub input: Value,
    pub context: Value,
    pub signal: Value,
}

#[derive(Debug, Clone, Copy)]
pub struct ScriptExecutionLimits {
    pub loop_iteration_limit: u64,
    pub recursion_limit: usize,
    pub stack_size_limit: usize,
    pub timeout_ms: u64,
}

impl ScriptExecutionLimits {
    pub fn for_logic() -> Self {
        Self {
            loop_iteration_limit: 1_000_000,
            recursion_limit: 128,
            stack_size_limit: 512,
            timeout_ms: 50,
        }
    }

    pub fn for_ui() -> Self {
        Self {
            loop_iteration_limit: 1_000_000,
            recursion_limit: 128,
            stack_size_limit: 512,
            timeout_ms: 200,
        }
    }
}

impl Default for ScriptExecutionLimits {
    fn default() -> Self {
        Self::for_ui()
    }
}

pub trait ScriptEngine: Send + Sync {
    fn execute(
        &self,
        script: &str,
        ctx: ScriptExecutionContext,
        limits: ScriptExecutionLimits,
    ) -> Result<Value>;
}

pub async fn execute_with_limits(
    engine: Arc<dyn ScriptEngine>,
    script: String,
    ctx: ScriptExecutionContext,
    limits: ScriptExecutionLimits,
) -> Result<Value> {
    let timeout_ms = limits.timeout_ms;
    let handle = tokio::task::spawn_blocking(move || engine.execute(&script, ctx, limits));

    if timeout_ms == 0 {
        return handle
            .await
            .map_err(|err| Error::System(format!("Script join failed: {}", err)))?;
    }

    match timeout(Duration::from_millis(timeout_ms), handle).await {
        Ok(result) => {
            result.map_err(|err| Error::System(format!("Script join failed: {}", err)))?
        }
        Err(_) => Err(Error::Validation("Script execution timed out".to_string())),
    }
}

#[derive(Default)]
pub struct BoaScriptEngine;

impl ScriptEngine for BoaScriptEngine {
    fn execute(
        &self,
        script: &str,
        ctx: ScriptExecutionContext,
        limits: ScriptExecutionLimits,
    ) -> Result<Value> {
        let input_json = serde_json::to_string(&ctx.input)
            .map_err(|err| Error::Validation(format!("Script input invalid: {}", err)))?;
        let context_json = serde_json::to_string(&ctx.context)
            .map_err(|err| Error::Validation(format!("Script context invalid: {}", err)))?;
        let signal_json = serde_json::to_string(&ctx.signal)
            .map_err(|err| Error::Validation(format!("Script signal invalid: {}", err)))?;

        let input_literal = serde_json::to_string(&input_json)
            .map_err(|err| Error::Validation(format!("Script input encoding failed: {}", err)))?;
        let context_literal = serde_json::to_string(&context_json)
            .map_err(|err| Error::Validation(format!("Script context encoding failed: {}", err)))?;
        let signal_literal = serde_json::to_string(&signal_json)
            .map_err(|err| Error::Validation(format!("Script signal encoding failed: {}", err)))?;

        let source = format!(
            r#"(function() {{
                const input = JSON.parse({input_literal});
                const context = JSON.parse({context_literal});
                const signal = JSON.parse({signal_literal});
                const handler = (function(input, context, signal) {{
{script}
                }});
                const result = handler(input, context, signal);
                return JSON.stringify(result === undefined ? null : result);
            }})()"#,
            input_literal = input_literal,
            context_literal = context_literal,
            signal_literal = signal_literal,
            script = script
        );

        let mut context = Context::default();
        {
            let runtime_limits = context.runtime_limits_mut();
            runtime_limits.set_loop_iteration_limit(limits.loop_iteration_limit);
            runtime_limits.set_recursion_limit(limits.recursion_limit);
            runtime_limits.set_stack_size_limit(limits.stack_size_limit);
        }
        let value = context
            .eval(Source::from_bytes(source.as_bytes()))
            .map_err(|err| Error::Validation(format!("Script execution failed: {}", err)))?;
        let js_string = value
            .to_string(&mut context)
            .map_err(|err| Error::Validation(format!("Script result invalid: {}", err)))?;
        let result_string = js_string
            .to_std_string()
            .map_err(|err| Error::Validation(format!("Script result invalid: {}", err)))?;
        let parsed: Value = serde_json::from_str(&result_string)
            .map_err(|err| Error::Validation(format!("Script output invalid JSON: {}", err)))?;
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn boa_engine_executes_script_and_returns_json() {
        let engine = BoaScriptEngine;
        let result = engine
            .execute(
                "return { outcome: 'continue', output: input.value };",
                ScriptExecutionContext {
                    input: json!({ "value": "success" }),
                    context: json!({}),
                    signal: json!({ "type": "execute_script" }),
                },
                ScriptExecutionLimits::default(),
            )
            .expect("script execution failed");

        assert_eq!(
            result.get("outcome").and_then(Value::as_str),
            Some("continue")
        );
        assert_eq!(
            result.get("output").and_then(Value::as_str),
            Some("success")
        );
    }
}
