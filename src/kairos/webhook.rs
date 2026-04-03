//! GitHub Webhook 처리
//! 
//! PR 이벤트, CI 결과 등을 수신하고 처리

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

/// GitHub 이벤트 종류
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GitHubEvent {
    /// Pull Request 이벤트
    PullRequest {
        action: String,  // opened, closed, merged, review_requested
        number: u64,
        title: String,
        url: String,
        repo: String,
    },
    /// PR Review 이벤트
    PullRequestReview {
        action: String,  // submitted, edited, dismissed
        pr_number: u64,
        reviewer: String,
        state: String,   // approved, changes_requested, commented
        body: Option<String>,
    },
    /// CI/Check 이벤트
    CheckRun {
        action: String,  // created, completed
        name: String,
        status: String,  // queued, in_progress, completed
        conclusion: Option<String>,  // success, failure, cancelled
        pr_number: Option<u64>,
    },
    /// Issue 코멘트
    IssueComment {
        action: String,  // created, edited, deleted
        issue_number: u64,
        author: String,
        body: String,
    },
}

/// Webhook 핸들러
pub struct WebhookHandler {
    event_tx: mpsc::Sender<GitHubEvent>,
}

impl WebhookHandler {
    pub fn new(event_tx: mpsc::Sender<GitHubEvent>) -> Self {
        Self { event_tx }
    }

    /// Webhook 페이로드 파싱 및 전송
    pub async fn handle_payload(&self, event_type: &str, payload: &str) -> Result<()> {
        let event = self.parse_event(event_type, payload)?;
        if let Some(e) = event {
            self.event_tx.send(e).await?;
        }
        Ok(())
    }

    fn parse_event(&self, event_type: &str, payload: &str) -> Result<Option<GitHubEvent>> {
        let json: serde_json::Value = serde_json::from_str(payload)?;
        
        match event_type {
            "pull_request" => {
                let action = json["action"].as_str().unwrap_or("").to_string();
                let pr = &json["pull_request"];
                Ok(Some(GitHubEvent::PullRequest {
                    action,
                    number: pr["number"].as_u64().unwrap_or(0),
                    title: pr["title"].as_str().unwrap_or("").to_string(),
                    url: pr["html_url"].as_str().unwrap_or("").to_string(),
                    repo: json["repository"]["full_name"].as_str().unwrap_or("").to_string(),
                }))
            }
            "pull_request_review" => {
                let action = json["action"].as_str().unwrap_or("").to_string();
                let review = &json["review"];
                Ok(Some(GitHubEvent::PullRequestReview {
                    action,
                    pr_number: json["pull_request"]["number"].as_u64().unwrap_or(0),
                    reviewer: review["user"]["login"].as_str().unwrap_or("").to_string(),
                    state: review["state"].as_str().unwrap_or("").to_string(),
                    body: review["body"].as_str().map(|s| s.to_string()),
                }))
            }
            "check_run" => {
                let action = json["action"].as_str().unwrap_or("").to_string();
                let check = &json["check_run"];
                Ok(Some(GitHubEvent::CheckRun {
                    action,
                    name: check["name"].as_str().unwrap_or("").to_string(),
                    status: check["status"].as_str().unwrap_or("").to_string(),
                    conclusion: check["conclusion"].as_str().map(|s| s.to_string()),
                    pr_number: check["pull_requests"]
                        .as_array()
                        .and_then(|arr| arr.first())
                        .and_then(|pr| pr["number"].as_u64()),
                }))
            }
            "issue_comment" => {
                let action = json["action"].as_str().unwrap_or("").to_string();
                let comment = &json["comment"];
                Ok(Some(GitHubEvent::IssueComment {
                    action,
                    issue_number: json["issue"]["number"].as_u64().unwrap_or(0),
                    author: comment["user"]["login"].as_str().unwrap_or("").to_string(),
                    body: comment["body"].as_str().unwrap_or("").to_string(),
                }))
            }
            _ => Ok(None),
        }
    }
}

/// GitHub 이벤트를 사람이 읽기 좋은 메시지로 변환
pub fn format_event(event: &GitHubEvent) -> String {
    match event {
        GitHubEvent::PullRequest { action, number, title, repo, .. } => {
            format!("🔀 PR #{} {} in {}: {}", number, action, repo, title)
        }
        GitHubEvent::PullRequestReview { pr_number, reviewer, state, .. } => {
            let emoji = match state.as_str() {
                "approved" => "✅",
                "changes_requested" => "🔄",
                _ => "💬",
            };
            format!("{} PR #{} {} by {}", emoji, pr_number, state, reviewer)
        }
        GitHubEvent::CheckRun { name, status, conclusion, pr_number, .. } => {
            let emoji = match conclusion.as_deref() {
                Some("success") => "✅",
                Some("failure") => "❌",
                _ => "⏳",
            };
            let pr_str = pr_number.map(|n| format!(" (PR #{})", n)).unwrap_or_default();
            format!("{} CI {}: {}{}", emoji, name, status, pr_str)
        }
        GitHubEvent::IssueComment { issue_number, author, body, .. } => {
            let preview = if body.len() > 50 { &body[..50] } else { body };
            format!("💬 #{} comment by {}: {}...", issue_number, author, preview)
        }
    }
}
