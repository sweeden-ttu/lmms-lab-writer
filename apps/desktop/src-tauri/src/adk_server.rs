use futures_util::{StreamExt, SinkExt};
use warp::Filter;
use std::sync::Arc;
use tokio::sync::Mutex;
use adk_rust::agents::legal_writer::{IterativeLegalWriter, TrustworthyAgent};

pub async fn start_adk_server() {
    let adk_agent = Arc::new(IterativeLegalWriter::new());

    let ws_route = warp::path("adk-eval")
        .and(warp::ws())
        .and(with_agent(adk_agent))
        .map(|ws: warp::ws::Ws, agent| {
            ws.on_upgrade(move |socket| handle_connection(socket, agent))
        });

    println!("[adk-rust] Server starting on ws://127.0.0.1:3030/adk-eval");
    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

fn with_agent(
    agent: Arc<IterativeLegalWriter>,
) -> impl Filter<Extract = (Arc<IterativeLegalWriter>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || agent.clone())
}

async fn handle_connection(ws: warp::ws::WebSocket, agent: Arc<IterativeLegalWriter>) {
    let (mut tx, mut rx) = ws.split();

    println!("[adk-rust] Chrome extension connected");

    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    println!("[adk-rust] Received payload: {}", text);
                    // Mock evaluation
                    let is_complete = agent.evaluate_completeness("local docs", text).await;
                    let response = format!("{{\"status\":\"evaluated\", \"complete\": {}}}", is_complete);
                    let _ = tx.send(warp::ws::Message::text(response)).await;
                }
            }
            Err(e) => {
                eprintln!("[adk-rust] WebSocket error: {}", e);
                break;
            }
        }
    }
}
