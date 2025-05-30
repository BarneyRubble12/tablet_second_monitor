use crate::error::AppError;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use warp::ws::{Message, WebSocket};
use serde_json::json;
use tokio::sync::mpsc;

pub struct WebSocketServer {
    tx: broadcast::Sender<Vec<u8>>,
}

impl WebSocketServer {
    pub fn new(tx: broadcast::Sender<Vec<u8>>) -> Self {
        Self { tx }
    }

    pub async fn handle_connection(&self, ws: WebSocket) {
        let (ws_sender, mut ws_receiver) = ws.split();
        let mut rx = self.tx.subscribe();

        // Create a channel for sending messages back to the WebSocket
        let (tx, mut rx_outgoing) = mpsc::channel::<Message>(100);
        let ws_sender = tokio::sync::Mutex::new(ws_sender);

        // Spawn a task to handle incoming messages
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Some(result) = ws_receiver.next().await {
                match result {
                    Ok(msg) => {
                        if msg.is_text() {
                            if let Ok(text) = msg.to_str() {
                                if let Ok(data) = serde_json::from_str::<serde_json::Value>(text) {
                                    if let Some(msg_type) = data.get("type").and_then(|t| t.as_str()) {
                                        match msg_type {
                                            "ping" => {
                                                if let Some(timestamp) = data.get("timestamp").and_then(|t| t.as_i64()) {
                                                    let pong = json!({
                                                        "type": "pong",
                                                        "timestamp": timestamp
                                                    });
                                                    if let Ok(pong_msg) = serde_json::to_string(&pong) {
                                                        let _ = tx_clone.send(Message::text(pong_msg)).await;
                                                    }
                                                }
                                            }
                                            _ => {
                                                // Handle other message types
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        });

        // Spawn a task to handle outgoing messages from the broadcast channel
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Ok(data) = rx.recv().await {
                if tx_clone.send(Message::binary(data)).await.is_err() {
                    break;
                }
            }
        });

        // Main task to forward messages to the WebSocket
        while let Some(msg) = rx_outgoing.recv().await {
            if let Err(_) = ws_sender.lock().await.send(msg).await {
                break;
            }
        }
    }
}

async fn serve_index() -> &'static str {
    include_str!("../../static/index.html")
} 