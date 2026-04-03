//! 🤖 KAIROS - 자율 에이전트 모드

pub mod auto_dream;
pub mod config_file;
pub mod consolidation;
pub mod daily_log;
pub mod daemon;
pub mod memdir;
pub mod notify;
pub mod server;
pub mod webhook;

pub use auto_dream::AutoDream;
pub use config_file::ConfigFile;
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
    pub min_hours: u64,
    pub min_sessions: u64,
    pub memory_path: std::path::PathBuf,
    pub imessage_recipient: Option<String>,
    pub telegram: Option<TelegramConfig>,
    pub discord_webhook: Option<String>,
    pub github_webhook_secret: Option<String>,
    pub server_port: u16,
}

#[derive(Debug, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

impl Default for KairosConfig {
    fn default() -> Self {
        // 설정 파일에서 로드 시도
        ConfigFile::load()
            .map(|c| c.to_kairos_config())
            .unwrap_or_else(|_| {
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
            })
    }
}
