//! HTTP 서버 - GitHub Webhook 수신
//! 
//! Endpoints:
//! - POST /webhook/github - GitHub Webhook 수신
//! - GET /health - 헬스 체크
//! - GET /status - KAIROS 상태

use super::{GitHubEvent, KairosConfig, Notification, Notifier, Priority, WebhookHandler, format_event};
use anyhow::Result;
use axum::{
    extract::{State, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;
use tokio::sync::mpsc;

type HmacSha256 = Hmac<Sha256>;

/// 서버 상태
pub struct ServerState {
    pub config: KairosConfig,
    pub event_tx: mpsc::Sender<GitHubEvent>,
    pub notifier: Arc<Notifier>,
}

/// HTTP 서버 시작
pub async fn start_server(
    config: KairosConfig,
    event_tx: mpsc::Sender<GitHubEvent>,
    notifier: Arc<Notifier>,
) -> Result<()> {
    let state = Arc::new(ServerState {
        config: config.clone(),
        event_tx,
        notifier,
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/webhook/github", post(github_webhook))
        .with_state(state);

    let port = 3847; // KAIROS port
    let addr = format!("0.0.0.0:{}", port);
    
    println!("🌐 KAIROS HTTP server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// 헬스 체크
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "kairos"
    }))
}

/// KAIROS 상태
async fn status(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "running",
        "memory_path": state.config.memory_path,
        "min_hours": state.config.min_hours,
        "min_sessions": state.config.min_sessions
    }))
}

/// GitHub Webhook 처리
async fn github_webhook(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    // Signature 검증
    if let Some(secret) = &state.config.github_webhook_secret {
        let signature = headers
            .get("X-Hub-Signature-256")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        
        if !verify_signature(secret, &body, signature) {
            return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
        }
    }

    // 이벤트 타입
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    println!("📥 GitHub webhook: {}", event_type);

    // 이벤트 파싱
    let handler = WebhookHandler::new(state.event_tx.clone());
    match handler.handle_payload(event_type, &body).await {
        Ok(_) => (StatusCode::OK, "OK").into_response(),
        Err(e) => {
            eprintln!("Webhook error: {}", e);
            (StatusCode::BAD_REQUEST, "Parse error").into_response()
        }
    }
}

/// GitHub Webhook Signature 검증
fn verify_signature(secret: &str, payload: &str, signature: &str) -> bool {
    let signature = signature.strip_prefix("sha256=").unwrap_or(signature);
    
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    
    mac.update(payload.as_bytes());
    
    let expected = hex::encode(mac.finalize().into_bytes());
    expected == signature
}
