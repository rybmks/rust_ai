use std::sync::Arc;

use axum::response::IntoResponse;
use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    routing::get,
};
use backend::{
    models::{Model, mistral7b::Mistral7B},
    *,
};
use candle_core::{Device, MetalDevice, backend::BackendDevice};
use futures::StreamExt;
use hf_hub::api::tokio::ApiBuilder;
use tokio::sync::Mutex;

const SAMPLE_LEN: usize = 100;

#[derive(Debug, Clone)]
pub struct AppState<M: Model + Send + Sync + 'static> {
    pub model: Arc<Mutex<M>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let token = "TOKEN";

    let api = ApiBuilder::new()
        .with_token(Some(token.to_string()))
        .with_cache_dir(std::path::PathBuf::from("./.hf_cache"))
        .build()?;
    let device = Device::Metal(MetalDevice::new(0)?);

    let model = Mistral7B::init(api, device).await?;

    let state = Arc::new(AppState {
        model: Arc::new(Mutex::new(model)),
    });

    let app = Router::new().route("/ws", get(handler)).with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState<impl Model + Send + Sync>>>,
) -> impl IntoResponse {
    tracing::info!("WebSocket upgrade requested");
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(
    mut socket: WebSocket,
    state: Arc<AppState<impl Model + Send + Sync + 'static>>,
) {
    tracing::info!("WebSocket connection established");

    let prompt = match socket.next().await {
        Some(Ok(Message::Text(prompt))) => {
            tracing::info!("Received prompt: {}", prompt);
            prompt
        }
        _ => {
            tracing::warn!("WebSocket closed before receiving prompt");
            let _ = socket.send(Message::Close(None)).await;
            return;
        }
    };

    let _ = socket
        .send(Message::Text(format!("Got your prompt: {prompt}").into()))
        .await;

    tracing::info!("Locking model for inference");
    let mut model = state.model.lock().await;

    let mut stream = {
        match model.run(prompt.to_string(), SAMPLE_LEN) {
            Ok(s) => {
                tracing::info!("Model run started");
                s
            }
            Err(e) => {
                tracing::error!("Model run error: {}", e);
                let _ = socket
                    .send(Message::Text(format!("Error: {e}").into()))
                    .await;
                let _ = socket.send(Message::Close(None)).await;
                return;
            }
        }
    };
    let mut count: usize = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(token) => {
                count += 1;
                tracing::info!("{count} Generated token: {}", token);
                if token.trim().is_empty() {
                    continue;
                }

                if socket
                    .send(Message::Text(token.clone().into()))
                    .await
                    .is_err()
                {
                    tracing::warn!("WebSocket send error, closing connection");
                    break;
                }
            }
            Err(e) => {
                let message = format!("Stream error: {e}");
                tracing::error!("{message}");
                let _ = socket.send(Message::Text(message.into())).await;
                break;
            }
        }
    }

    tracing::info!("Closing WebSocket connection");
    let _ = socket.send(Message::Close(None)).await;
}
