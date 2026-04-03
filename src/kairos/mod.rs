//! 🤖 KAIROS - 자율 에이전트 모드
//! 
//! Claude Code의 KAIROS를 Rust로 구현
//! - 백그라운드 데몬 실행
//! - 자동 메모리 정리 (Auto Dream)
//! - 일별 로그 시스템
//! - GitHub Webhook 처리
//! - 푸시 알림
//! - HTTP 서버

pub mod auto_dream;
pub mod consolidation;
pub mod daily_log;
pub mod daemon;
pub mod memdir;
pub mod notify;
pub mod server;
pub mod webhook;

pub use auto_dream::AutoDream;
pub use consolidation::ConsolidationLock;
pub use daily_log::DailyLog;
pub use daemon::{DaemonStatus, KairosDaemon};
pub use memdir::MemoryDir;
pub use notify::{Notification, Notifier, Priority};
pub use server::start_server;
pub use webhook::{GitHubEvent, WebhookHandler, format_event};

/// KAIROS 설정
#[derive(Debug, Clone)]
pub struct KairosConfig {
    /// 메모리 정리 최소 간격 (시간)
    pub min_hours: u64,
    /// 메모리 정리 최소 세션 수
    pub min_sessions: u64,
    /// 메모리 디렉토리 경로
    pub memory_path: std::path::PathBuf,
    /// iMessage 수신자
    pub imessage_recipient: Option<String>,
    /// Telegram 설정
    pub telegram: Option<TelegramConfig>,
    /// Discord Webhook URL
    pub discord_webhook: Option<String>,
    /// GitHub Webhook Secret
    pub github_webhook_secret: Option<String>,
    /// HTTP 서버 포트
    pub server_port: u16,
}

#[derive(Debug, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

impl Default for KairosConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            min_hours: 24,
            min_sessions: 5,
            memory_path: home.join(".claude-sam").join("memory"),
            imessage_recipient: None,
            telegram: None,
            discord_webhook: None,
            github_webhook_secret: None,
            server_port: 3847,
        }
    }
}
