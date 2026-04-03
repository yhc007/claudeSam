//! 설정 파일 관리 (config.toml)
//! 
//! 위치: ~/.claude-sam/config.toml

use super::{KairosConfig, TelegramConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 설정 파일 구조
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub kairos: KairosSection,
    #[serde(default)]
    pub notifications: NotificationsSection,
    #[serde(default)]
    pub github: GitHubSection,
    #[serde(default)]
    pub server: ServerSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KairosSection {
    /// 메모리 정리 최소 간격 (시간)
    #[serde(default = "default_min_hours")]
    pub min_hours: u64,
    /// 메모리 정리 최소 세션 수
    #[serde(default = "default_min_sessions")]
    pub min_sessions: u64,
    /// 메모리 디렉토리 경로 (기본: ~/.claude-sam/memory)
    pub memory_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationsSection {
    /// iMessage 수신자 (전화번호 또는 이메일)
    pub imessage: Option<String>,
    /// Telegram 설정
    pub telegram: Option<TelegramSection>,
    /// Discord Webhook URL
    pub discord_webhook: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramSection {
    pub bot_token: String,
    pub chat_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubSection {
    /// Webhook Secret
    pub webhook_secret: Option<String>,
    /// 감시할 저장소 목록
    #[serde(default)]
    pub repos: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSection {
    /// HTTP 서버 포트
    #[serde(default = "default_port")]
    pub port: u16,
    /// 바인드 주소
    #[serde(default = "default_bind")]
    pub bind: String,
}

fn default_min_hours() -> u64 { 24 }
fn default_min_sessions() -> u64 { 5 }
fn default_port() -> u16 { 3847 }
fn default_bind() -> String { "0.0.0.0".to_string() }

impl Default for KairosSection {
    fn default() -> Self {
        Self {
            min_hours: default_min_hours(),
            min_sessions: default_min_sessions(),
            memory_path: None,
        }
    }
}

impl Default for ServerSection {
    fn default() -> Self {
        Self {
            port: default_port(),
            bind: default_bind(),
        }
    }
}

impl ConfigFile {
    /// 설정 파일 경로
    pub fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".claude-sam")
            .join("config.toml")
    }

    /// 설정 파일 로드
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: ConfigFile = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// 설정 파일 저장
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// 기본 설정 파일 생성
    pub fn create_default() -> Result<()> {
        let config = Self::default();
        config.save()?;
        println!("✅ Created config file: {:?}", Self::config_path());
        Ok(())
    }

    /// KairosConfig로 변환
    pub fn to_kairos_config(&self) -> KairosConfig {
        let home = dirs::home_dir().unwrap_or_default();
        let memory_path = self.kairos.memory_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".claude-sam").join("memory"));

        KairosConfig {
            min_hours: self.kairos.min_hours,
            min_sessions: self.kairos.min_sessions,
            memory_path,
            imessage_recipient: self.notifications.imessage.clone(),
            telegram: self.notifications.telegram.as_ref().map(|t| TelegramConfig {
                bot_token: t.bot_token.clone(),
                chat_id: t.chat_id.clone(),
            }),
            discord_webhook: self.notifications.discord_webhook.clone(),
            github_webhook_secret: self.github.webhook_secret.clone(),
            server_port: self.server.port,
        }
    }
}
