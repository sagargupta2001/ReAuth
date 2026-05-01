use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpDeliveryRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct HttpDeliveryResponse {
    pub status_code: u16,
    pub body: String,
}

#[derive(Debug)]
pub struct HttpDeliveryError {
    pub message: String,
    pub error_chain: Vec<String>,
}

impl std::fmt::Display for HttpDeliveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for HttpDeliveryError {}

#[async_trait]
pub trait HttpDeliveryClient: Send + Sync {
    async fn send(
        &self,
        request: HttpDeliveryRequest,
    ) -> Result<HttpDeliveryResponse, HttpDeliveryError>;
}
