use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    Json,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::types::{ClientMessage, UserInput, WsResponse};

pub struct ServerState {
    pub tx_chat: broadcast::Sender<ClientMessage>,
    pub tx_screencast: broadcast::Sender<String>,
    pub tx_agent: broadcast::Sender<WsResponse>,
}

use tower_http::cors::{Any, CorsLayer};

pub async fn start_server(state: Arc<ServerState>) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/chat", get(chat_handler))
        .route("/screencast", get(screencast_handler))
        .route("/health", get(health_handler))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:44444").await.unwrap();
    println!("🚀 Server listening on http://0.0.0.0:44444");
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

pub(crate) fn parse_client_message(text: &str) -> Result<ClientMessage, serde_json::Error> {
    match serde_json::from_str::<ClientMessage>(text) {
        Ok(message) => Ok(message),
        Err(primary_err) => match serde_json::from_str::<UserInput>(text) {
            Ok(input) => Ok(ClientMessage::UserInput {
                history: input.history,
                message: input.message,
            }),
            Err(_) => Err(primary_err),
        },
    }
}

async fn chat_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    println!("🔌 Chat WebSocket Connection Attempt...");
    ws.on_upgrade(|socket| handle_chat_socket(socket, state))
}

async fn handle_chat_socket(socket: WebSocket, state: Arc<ServerState>) {
    println!("✅ Chat WebSocket Upgraded!");
    let (mut sender, mut receiver) = socket.split();

    // Spawn a task to forward agent responses to the UI
    let mut rx_agent = state.tx_agent.subscribe();
    tokio::spawn(async move {
        loop {
            match rx_agent.recv().await {
                Ok(response) => {
                    let msg = serde_json::to_string(&response).unwrap_or_default();
                    if !msg.is_empty() {
                        if sender.send(Message::Text(msg)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    // Buffer overflow: we fell behind. Log it and keep reading from newest.
                    println!("⚠️ WS broadcast lagged, skipped {} messages", skipped);
                }
                Err(_) => break, // Channel closed
            }
        }
    });

    // Handle incoming user messages from UI
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            println!("📥 Received message from UI: {}", text);
            if let Ok(input) = parse_client_message(&text) {
                let _ = state.tx_chat.send(input);
            }
        }
    }
}

async fn screencast_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    println!("🔌 Screencast WebSocket Connection Attempt...");
    ws.on_upgrade(|socket| handle_screencast_socket(socket, state))
}

async fn handle_screencast_socket(socket: WebSocket, state: Arc<ServerState>) {
    println!("✅ Screencast WebSocket Upgraded!");
    let (mut sender, _) = socket.split();
    let mut rx = state.tx_screencast.subscribe();

    loop {
        match rx.recv().await {
            Ok(frame) => {
                if sender.send(Message::Text(frame)).await.is_err() {
                    break; // client disconnected
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                // Frames are disposable — just skip and keep the socket alive
                println!("📺 Screencast lagged {} frames, catching up", n);
            }
            Err(_) => break, // channel closed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_client_message;
    use crate::types::{ClientMessage, ControlCommand, ControlSource};

    #[test]
    fn parses_enveloped_user_input() {
        let message = parse_client_message(r#"{"kind":"user_input","history":[],"message":"hello"}"#).unwrap();
        match message {
            ClientMessage::UserInput { message, .. } => assert_eq!(message, "hello"),
            _ => panic!("expected user input"),
        }
    }

    #[test]
    fn parses_legacy_user_input() {
        let message = parse_client_message(r#"{"history":[],"message":"hello"}"#).unwrap();
        match message {
            ClientMessage::UserInput { message, .. } => assert_eq!(message, "hello"),
            _ => panic!("expected user input"),
        }
    }

    #[test]
    fn parses_control_message() {
        let message = parse_client_message(r#"{"kind":"control","command":"stop","source":"voice"}"#).unwrap();
        match message {
            ClientMessage::Control { command, source } => {
                assert_eq!(command, ControlCommand::Stop);
                assert_eq!(source, ControlSource::Voice);
            }
            _ => panic!("expected control message"),
        }
    }
}
