mod support;

#[path = "api/health.rs"]
mod health;

#[path = "api/jwks.rs"]
mod jwks;

#[path = "api/auth_oidc_flow.rs"]
mod auth_oidc_flow;

#[path = "api/request_id.rs"]
mod request_id;
