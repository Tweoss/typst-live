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
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
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
    // Typst ouputs in the form 001.svg if there are up to 999 files.
    // The frontend doesn't know how many files are generated, so we
    // normalize here.
    fn normalize(s: &str) -> Option<String> {
        let s = s.strip_prefix("output_")?;
        let s = s.strip_suffix(".svg")?;
        Some(format!("output_{}.svg", s.trim_start_matches('0')))
    }
    let target = normalize(&svg_file).unwrap_or(svg_file);

    // We search for a file with the same normalized format.
    let filename = state
        .directory
        .path()
        .read_dir()
        .expect("Could not read directory")
        .find_map(|f| {
            let f = f.ok()?;
            if normalize(&f.file_name().to_string_lossy()).as_ref() == Some(&target) {
                Some(f.path())
            } else {
                None
            }
        })
        .unwrap_or(state.directory.path().join(target));

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
    let mut last_update: HashMap<usize, Instant> = HashMap::new();
    loop {
        // Implement basic debouncing per 100ms.
        if let Ok(index) = receiver.recv().await {
            let last = last_update.insert(index, Instant::now());
            if last.is_none()
                || last.is_some_and(|last| {
                    Instant::now().duration_since(last) > Duration::from_millis(10)
                })
            {
                println!("[INFO] refreshing page {index}");
                _ = socket.send(Message::Text(format!("refresh:{index}"))).await;
            }
        }
    }
}
