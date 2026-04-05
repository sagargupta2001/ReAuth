use crate::domain::flow::provider::NodeProvider;
use crate::domain::ui::UiSurface;
use serde_json::{json, Value};

pub struct ScriptedUiNodeProvider;

impl NodeProvider for ScriptedUiNodeProvider {
    fn id(&self) -> &'static str {
        "core.ui.scripted"
    }

    fn display_name(&self) -> &'static str {
        "Scripted UI"
    }

    fn description(&self) -> &'static str {
        "Handle custom UI signals with a server-side script."
    }

    fn icon(&self) -> &'static str {
        "Zap"
    }

    fn category(&self) -> &'static str {
        "Authenticator"
    }

    fn outputs(&self) -> Vec<&'static str> {
        vec!["success", "failure"]
    }

    fn config_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "screen_id": {
                    "type": "string",
                    "title": "Screen ID",
                    "default": "core.ui.scripted"
                },
                "ui_context": {
                    "type": "object",
                    "title": "UI Context",
                    "default": {}
                },
                "script": {
                    "type": "string",
                    "title": "Default Script",
                    "description": "Handler body. Receives (input, context, signal). Return { outcome, output?, context? }.",
                    "format": "code"
                },
                "signal_handlers": {
                    "type": "object",
                    "title": "Signal Handlers",
                    "additionalProperties": {
                        "type": "string"
                    }
                }
            },
            "required": ["script"]
        })
    }

    fn supports_ui(&self) -> bool {
        true
    }

    fn ui_surface(&self) -> Option<UiSurface> {
        Some(UiSurface::Form)
    }
}
