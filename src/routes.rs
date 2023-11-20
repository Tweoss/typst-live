use crate::state::ServerState;
use axum::{
    body::Full,
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::{header::CONTENT_TYPE, Response, StatusCode},
    response::{Html, IntoResponse},
};
use std::sync::Arc;
use tokio::fs;

pub async fn root(State(state): State<Arc<ServerState>>) -> Html<String> {
    include_str!("index.html")
        .replace("{addr}", &state.args.address)
        .replace("{port}", &state.args.port.to_string())
        .into()
}

pub async fn target(
    Path(svg_file): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    let filename = state.directory.path().join(svg_file);

    match fs::read(&filename).await {
        Ok(data) => Response::builder()
            .header(CONTENT_TYPE, "image/svg+xml")
            .body(Full::from(data))
            .expect("Failed to build response"),
        Err(err) => {
            println!("[INFO] Failed to read `{}` {err:?}", filename.display());
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::from("file not found"))
                .expect("Failed to build response")
        }
    }
}

pub async fn listen(
    State(state): State<Arc<ServerState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handler(socket, state))
}

async fn handler(mut socket: WebSocket, state: Arc<ServerState>) {
    let mut receiver = state.changed.subscribe();
    loop {
        if let Ok(index) = receiver.recv().await {
            _ = socket.send(Message::Text(format!("refresh:{index}"))).await;
        }
    }
}
