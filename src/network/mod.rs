use crate::error::AppError;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use warp::ws::{Message, WebSocket};

pub struct WebSocketServer {
    tx: broadcast::Sender<Vec<u8>>,
}

impl WebSocketServer {
    pub fn new(tx: broadcast::Sender<Vec<u8>>) -> Self {
        Self { tx }
    }

    pub async fn handle_connection(&self, ws: WebSocket) {
        let (mut ws_sender, _) = ws.split();
        let mut rx = self.tx.subscribe();

        while let Ok(data) = rx.recv().await {
            if ws_sender.send(Message::binary(data)).await.is_err() {
                break;
            }
        }
    }
}

async fn serve_index() -> &'static str {
    include_str!("../../static/index.html")
} 