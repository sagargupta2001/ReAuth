use axum::{
    body::Body,
    extract::{FromRequest, Json as AxumJson},
    http::Request,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::{de::DeserializeOwned, Serialize};
use std::borrow::Cow;
use std::fmt;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

/// Wrapper for validated JSON requests
#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S, Body> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send + 'static,
    S: Send + Sync,
{
    type Rejection = ValidationRejection;

    fn from_request(
        req: Request<Body>,
        state: &S,
    ) -> impl futures::Future<Output = Result<Self, <Self as FromRequest<S, Body>>::Rejection>> + Send
    {
        Box::pin(async move {
            // Try to parse the JSON
            let result = AxumJson::<T>::from_request(req, state).await;

            let AxumJson(value) = match result {
                Ok(json) => json,
                Err(rejection) => {
                    // Try to detect missing field errors from serde
                    let msg = rejection.to_string();
                    if let Some(field) = extract_missing_field(&msg) {
                        let mut errors = ValidationErrors::new();
                        let mut err = ValidationError::new("required");
                        err.message = Some(format!("{} field is required", field).into());

                        // Manually insert to own the field key and avoid lifetime issues
                        errors.errors_mut().insert(
                            Cow::from(field.to_string()),
                            ValidationErrorsKind::Field(vec![err]),
                        );

                        return Err(ValidationRejection::Validation(errors));
                    }

                    // Otherwise treat as a general JSON error
                    return Err(ValidationRejection::Json(msg));
                }
            };

            // Run validator checks
            value.validate().map_err(ValidationRejection::Validation)?;

            Ok(Self(value))
        })
    }
}

/// Unified error type for JSON and validation failures
#[derive(Debug, Serialize)]
#[serde(tag = "error", content = "details")]
pub enum ValidationRejection {
    Json(String),
    Validation(validator::ValidationErrors),
}

impl fmt::Display for ValidationRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(msg) => write!(f, "Invalid JSON: {}", msg),
            Self::Validation(_) => write!(f, "Validation failed"),
        }
    }
}

impl IntoResponse for ValidationRejection {
    fn into_response(self) -> Response {
        match self {
            Self::Json(msg) => {
                let body = AxumJson(serde_json::json!({
                    "error": "Invalid JSON",
                    "message": msg
                }));
                (StatusCode::BAD_REQUEST, body).into_response()
            }
            Self::Validation(errors) => {
                // Flatten validation errors to a simpler JSON structure
                let mut flat = serde_json::Map::new();
                for (field, errs) in errors.field_errors() {
                    if let Some(first) = errs.first() {
                        if let Some(msg) = &first.message {
                            flat.insert(field.to_string(), serde_json::json!(msg));
                        } else {
                            flat.insert(field.to_string(), serde_json::json!(first.code));
                        }
                    }
                }

                let body = AxumJson(serde_json::json!({
                    "error": "Validation failed",
                    "fields": flat
                }));
                (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
            }
        }
    }
}

/// Try to extract the missing field name from serde's error message
fn extract_missing_field(msg: &str) -> Option<String> {
    if let Some(start) = msg.find("missing field `") {
        let rest = &msg[start + "missing field `".len()..];
        if let Some(end) = rest.find('`') {
            return Some(rest[..end].to_string());
        }
    }
    None
}
