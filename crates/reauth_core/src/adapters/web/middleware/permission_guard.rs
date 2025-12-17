use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub async fn require_permission(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
    // You must pass the required permission string via a closure or wrapper in the router
    required_permission: &str,
) -> Response {
    // 1. Extract User ID from the request extensions (set by AuthMiddleware)
    let user_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap()
        }
    };

    // 2. Check Permission via Service (which uses Cache)
    match state
        .rbac_service
        .user_has_permission(&user_id, required_permission)
        .await
    {
        Ok(true) => next.run(req).await,
        _ => Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Insufficient Permissions"))
            .unwrap(),
    }
}
