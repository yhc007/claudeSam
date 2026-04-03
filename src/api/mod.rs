use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;

/// Claude Code CLI를 백엔드로 사용하는 클라이언트
/// Max 플랜 구독으로 API 키 없이 사용 가능!
#[derive(Clone)]
pub struct ClaudeClient {
    cli_path: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
}

#[derive(Serialize, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub struct MessageResponse {
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
}

impl ClaudeClient {
    pub fn new() -> Self {
        Self {
            cli_path: std::env::var("CLAUDE_CLI_PATH")
                .unwrap_or_else(|_| "claude".to_string()),
        }
    }

    /// Claude Code CLI를 호출하여 응답 받기
    pub async fn send_message(
        &self,
        messages: Vec<Message>,
        _tools: Option<Vec<Tool>>,
    ) -> Result<MessageResponse> {
        // 마지막 사용자 메시지 추출
        let last_user_msg = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .and_then(|m| match &m.content {
                MessageContent::Text(t) => Some(t.clone()),
                MessageContent::Blocks(blocks) => blocks
                    .iter()
                    .find(|b| b.block_type == "text")
                    .and_then(|b| b.text.clone()),
            })
            .unwrap_or_default();

        // Claude Code CLI 호출 (--print: 출력만, --dangerously-skip-permissions: 도구 자동 승인)
        let output = Command::new(&self.cli_path)
            .args([
                "--print",
                "--dangerously-skip-permissions",
                &last_user_msg,
            ])
            .output()?;

        let response_text = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !output.status.success() && response_text.is_empty() {
            return Ok(MessageResponse {
                content: vec![ContentBlock {
                    block_type: "text".to_string(),
                    text: Some(format!("Error: {}", stderr)),
                    id: None,
                    name: None,
                    input: None,
                }],
                stop_reason: Some("error".to_string()),
            });
        }

        Ok(MessageResponse {
            content: vec![ContentBlock {
                block_type: "text".to_string(),
                text: Some(response_text),
                id: None,
                name: None,
                input: None,
            }],
            stop_reason: Some("end_turn".to_string()),
        })
    }
}

impl Default for ClaudeClient {
    fn default() -> Self {
        Self::new()
    }
}
