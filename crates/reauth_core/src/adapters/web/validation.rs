use axum::{
    body::Body,
    extract::{FromRequest, Json as AxumJson},
    http::Request,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;
use validator::Validate;

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
            // Extract JSON body
            let AxumJson(value) = AxumJson::<T>::from_request(req, state)
                .await
                .map_err(|rejection| ValidationRejection::Json(rejection.to_string()))?;

            // Validate using `validator`
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
                let body = AxumJson(serde_json::json!({
                    "error": "Validation failed",
                    "fields": errors.field_errors()
                }));
                (StatusCode::UNPROCESSABLE_ENTITY, body).into_response()
            }
        }
    }
}
