//! 푸시 알림 시스템
//! 
//! 다양한 채널로 알림 전송:
//! - macOS 알림 (terminal-notifier)
//! - iMessage (imsg CLI)
//! - Telegram
//! - Discord Webhook

use anyhow::Result;
use std::process::Command;

/// 알림 우선순위
#[derive(Debug, Clone, Copy)]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

/// 알림 메시지
#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub priority: Priority,
    pub url: Option<String>,
}

impl Notification {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            priority: Priority::Normal,
            url: None,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}

/// 알림 전송자
pub struct Notifier {
    /// iMessage 수신자 (전화번호 또는 이메일)
    imessage_recipient: Option<String>,
    /// Telegram Bot Token
    telegram_token: Option<String>,
    /// Telegram Chat ID
    telegram_chat_id: Option<String>,
    /// Discord Webhook URL
    discord_webhook: Option<String>,
}

impl Notifier {
    pub fn new() -> Self {
        Self {
            imessage_recipient: None,
            telegram_token: None,
            telegram_chat_id: None,
            discord_webhook: None,
        }
    }

    pub fn with_imessage(mut self, recipient: impl Into<String>) -> Self {
        self.imessage_recipient = Some(recipient.into());
        self
    }

    pub fn with_telegram(mut self, token: impl Into<String>, chat_id: impl Into<String>) -> Self {
        self.telegram_token = Some(token.into());
        self.telegram_chat_id = Some(chat_id.into());
        self
    }

    pub fn with_discord(mut self, webhook_url: impl Into<String>) -> Self {
        self.discord_webhook = Some(webhook_url.into());
        self
    }

    /// 모든 설정된 채널로 알림 전송
    pub async fn send(&self, notif: &Notification) -> Result<()> {
        // macOS 알림 (항상)
        self.send_macos(notif)?;

        // iMessage
        if self.imessage_recipient.is_some() {
            let _ = self.send_imessage(notif).await;
        }

        // Telegram
        if self.telegram_token.is_some() && self.telegram_chat_id.is_some() {
            let _ = self.send_telegram(notif).await;
        }

        // Discord
        if self.discord_webhook.is_some() {
            let _ = self.send_discord(notif).await;
        }

        Ok(())
    }

    /// macOS 알림 (terminal-notifier 또는 osascript)
    fn send_macos(&self, notif: &Notification) -> Result<()> {
        // terminal-notifier 시도
        let result = Command::new("terminal-notifier")
            .args([
                "-title", &notif.title,
                "-message", &notif.body,
                "-sound", "default",
            ])
            .output();

        if result.is_err() {
            // osascript 폴백
            let script = format!(
                r#"display notification "{}" with title "{}""#,
                notif.body.replace('"', r#"\""#),
                notif.title.replace('"', r#"\""#)
            );
            Command::new("osascript")
                .args(["-e", &script])
                .output()?;
        }

        Ok(())
    }

    /// iMessage 전송 (imsg CLI 사용)
    async fn send_imessage(&self, notif: &Notification) -> Result<()> {
        let recipient = self.imessage_recipient.as_ref().unwrap();
        let message = format!("🤖 {}\n\n{}", notif.title, notif.body);
        
        Command::new("imsg")
            .args(["send", recipient, &message])
            .output()?;
        
        Ok(())
    }

    /// Telegram 전송
    async fn send_telegram(&self, notif: &Notification) -> Result<()> {
        let token = self.telegram_token.as_ref().unwrap();
        let chat_id = self.telegram_chat_id.as_ref().unwrap();
        let message = format!("🤖 *{}*\n\n{}", notif.title, notif.body);
        
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            token
        );
        
        let client = reqwest::Client::new();
        client.post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": message,
                "parse_mode": "Markdown"
            }))
            .send()
            .await?;
        
        Ok(())
    }

    /// Discord Webhook 전송
    async fn send_discord(&self, notif: &Notification) -> Result<()> {
        let webhook = self.discord_webhook.as_ref().unwrap();
        
        let client = reqwest::Client::new();
        client.post(webhook)
            .json(&serde_json::json!({
                "embeds": [{
                    "title": notif.title,
                    "description": notif.body,
                    "color": match notif.priority {
                        Priority::Low => 0x808080,
                        Priority::Normal => 0x0099ff,
                        Priority::High => 0xff9900,
                        Priority::Urgent => 0xff0000,
                    }
                }]
            }))
            .send()
            .await?;
        
        Ok(())
    }
}

impl Default for Notifier {
    fn default() -> Self {
        Self::new()
    }
}
