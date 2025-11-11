use crate::adapters::web::server::AppState;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::stream::StreamExt;
use manager::log_bus::LogSubscriber;
use std::sync::Arc;
use tracing::info;

/// Axum handler to upgrade a connection to a WebSocket.
pub async fn log_stream_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_log_socket(socket, state.log_subscriber))
}

/// The main WebSocket connection loop.
async fn handle_log_socket(mut socket: WebSocket, log_subscriber: Arc<dyn LogSubscriber>) {
    info!("New log stream client connected.");

    // Subscribe to the log bus. This gets a new receiver.
    let mut log_rx = log_subscriber.subscribe();

    loop {
        tokio::select! {
            // 1. Listen for new log messages from the bus
            Ok(log_entry) = log_rx.recv() => {
                let payload = serde_json::to_string(&log_entry).unwrap_or_default();
                if socket.send(Message::Text(Utf8Bytes::from(payload))).await.is_err() {
                    // Client disconnected, break the loop
                    info!("Log stream client disconnected.");
                    break;
                }
            }

            // 2. Listen for messages from the client (e.g., ping/pong or filter requests)
            Some(Ok(msg)) = socket.next() => {
                if let Message::Close(_) = msg {
                    info!("Log stream client disconnected.");
                    break;
                }
                // Here you could add logic to handle incoming filter messages
            }

            // 3. Client disconnected without a close message
            else => {
                info!("Log stream client disconnected.");
                break;
            }
        }
    }
}
