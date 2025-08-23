use futures_util::{SinkExt, StreamExt};
use mdgen::generator::MarketDataGenerator;
use serde_json;
// use std::collections::HashMap;
// use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast};
use tokio::time::interval;
// use tokio_tungstenite::{accept_async};
use warp::Filter;

// type Connections = Arc<Mutex<HashMap<usize, broadcast::Sender<String>>>>;

#[tokio::main]
async fn main() {
    let (tx, _rx ) = broadcast::channel::<String>(1000);
    let tx_clone = tx.clone();

    // Spawn market data generator task
    tokio::spawn(async move {
        let mut generator = MarketDataGenerator::new(100.0, 0.2, 0.01);
        let mut interval = interval(Duration::from_millis(100));
        
        loop {
            interval.tick().await;
            if let Some(data) = generator.next() {
                if let Ok(json) = serde_json::to_string(&data) {
                    let _ = tx_clone.send(json);
                }
            }
        }
    });

    // CORS
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    // WebSocket route
    let websocket = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || tx.clone()))
        .map(|ws: warp::ws::Ws, tx: broadcast::Sender<String>| {
            ws.on_upgrade(move |socket| handle_websocket(socket, tx))
        });

    // Static API route for fallback
    let api = warp::path("api")
        .and(warp::path("market-data"))
        .and(warp::get())
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "message": "Use WebSocket endpoint /ws for real-time data"
            }))
        });

    let routes = websocket.or(api).with(cors);

    println!("🚀 Market data server running at http://localhost:8080");
    println!("📡 WebSocket endpoint: ws://localhost:8080/ws");
    
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn handle_websocket(
    websocket: warp::ws::WebSocket,
    tx: broadcast::Sender<String>,
) {
    let (mut ws_tx, mut ws_rx) = websocket.split();
    let mut rx = tx.subscribe();

    // Send initial connection message
    let welcome = serde_json::json!({
        "type": "connected",
        "message": "Market data stream connected"
    });
    
    if ws_tx.send(warp::ws::Message::text(welcome.to_string())).await.is_err() {
        return;
    }

    // Spawn task to forward broadcast messages to WebSocket
    let tx_task = tokio::spawn(async move {
        while let Ok(data) = rx.recv().await {
            if ws_tx.send(warp::ws::Message::text(data)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming WebSocket messages (ping/pong, etc.)
    let rx_task = tokio::spawn(async move {
        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(msg) => {
                    if msg.is_close() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    tokio::select! {
        _ = tx_task => {},
        _ = rx_task => {},
    }
}