mod capture;
mod config;
mod error;
mod input;
mod network;
mod utils;

use capture::ScreenCapture;
use config::CaptureConfig;
use error::AppError;
use input::InputHandler;
use network::WebSocketServer;
use std::net::SocketAddr;
use tokio::sync::broadcast;
use warp::Filter;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::io::Cursor;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize configuration
    let config = CaptureConfig::default();

    // Create broadcast channel for frame data
    let (frame_tx, _) = broadcast::channel(2);
    let (input_tx, _) = broadcast::channel(100);

    // Initialize screen capture
    let capture = Arc::new(Mutex::new(ScreenCapture::new(config.clone())?));
    println!("Found {} displays", capture.lock().await.get_display_count());

    // Initialize input handler
    let _input_handler = InputHandler::new(input_tx);

    // Create static file server
    let static_dir = PathBuf::from("static");
    let static_files = warp::path("static").and(warp::fs::dir(static_dir));
    let index = warp::path::end().and(warp::fs::file("static/index.html"));
    
    // Create WebSocket route
    let capture_clone = capture.clone();
    let frame_tx_clone = frame_tx.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let capture = capture_clone.clone();
            let frame_tx = frame_tx_clone.clone();
            ws.on_upgrade(move |socket| async move {
                let server = WebSocketServer::new(frame_tx);
                server.handle_connection(socket).await;
            })
        });

    // Combine routes
    let routes = static_files.or(index).or(ws_route);

    // Start the HTTP server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Starting server on {}", addr);
    
    // Start capture loop
    let capture_clone = capture.clone();
    let frame_tx_clone = frame_tx.clone();
    let capture_handle = tokio::spawn(async move {
        loop {
            let mut capture = capture_clone.lock().await;
            match capture.capture_frame() {
                Ok(frame) => {
                    // Convert frame to JPEG
                    let mut buf = Vec::new();
                    let mut cursor = Cursor::new(&mut buf);
                    if let Ok(_) = frame.write_to(&mut cursor, image::ImageFormat::Jpeg) {
                        let _ = frame_tx_clone.send(buf);
                    }
                }
                Err(e) => eprintln!("Capture error: {}", e),
            }
            tokio::time::sleep(capture.frame_duration()).await;
        }
    });

    // Start warp server
    warp::serve(routes).run(addr).await;

    // Wait for capture loop
    if let Err(e) = capture_handle.await {
        eprintln!("Capture task error: {}", e);
    }

    Ok(())
} 