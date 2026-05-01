use crate::ports::http_client::{
    HttpDeliveryClient, HttpDeliveryError, HttpDeliveryRequest, HttpDeliveryResponse,
};
use async_trait::async_trait;
use reqwest::Client;
use std::error::Error as StdError;

pub struct ReqwestDeliveryClient {
    client: Client,
}

impl ReqwestDeliveryClient {
    pub fn new(timeout: std::time::Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }
}

#[async_trait]
impl HttpDeliveryClient for ReqwestDeliveryClient {
    async fn send(
        &self,
        request: HttpDeliveryRequest,
    ) -> Result<HttpDeliveryResponse, HttpDeliveryError> {
        let method = parse_http_method(&request.method);
        let mut req_builder = self.client.request(method, &request.url);

        for (k, v) in request.headers {
            req_builder = req_builder.header(&k, &v);
        }

        let response = req_builder.body(request.body).send().await;

        match response {
            Ok(resp) => {
                let status_code = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_default();
                Ok(HttpDeliveryResponse { status_code, body })
            }
            Err(err) => {
                let error_chain = collect_error_chain(&err);
                Err(HttpDeliveryError {
                    message: err.to_string(),
                    error_chain,
                })
            }
        }
    }
}

fn parse_http_method(method: &str) -> reqwest::Method {
    reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::POST)
}

fn collect_error_chain(error: &reqwest::Error) -> Vec<String> {
    let mut chain = Vec::new();
    let mut current: Option<&(dyn StdError + 'static)> = Some(error);
    while let Some(err) = current {
        chain.push(err.to_string());
        current = err.source();
    }
    chain
}
