//! KAIROS 백그라운드 데몬
//! 
//! 터미널을 닫아도 계속 실행되는 백그라운드 에이전트
//! - Auto Dream 스케줄링
//! - GitHub Webhook 처리
//! - 푸시 알림 전송

use super::{
    AutoDream, GitHubEvent, KairosConfig, Notification, Notifier, 
    Priority, WebhookHandler, format_event
};
use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

/// KAIROS 데몬
pub struct KairosDaemon {
    config: KairosConfig,
    pid_file: PathBuf,
    auto_dream: AutoDream,
    notifier: Notifier,
}

impl KairosDaemon {
    pub fn new(config: KairosConfig) -> Self {
        let pid_file = config.memory_path.join(".kairos.pid");
        let auto_dream = AutoDream::new(config.clone());
        
        // Notifier 설정
        let mut notifier = Notifier::new();
        if let Some(ref recipient) = config.imessage_recipient {
            notifier = notifier.with_imessage(recipient);
        }
        if let Some(ref tg) = config.telegram {
            notifier = notifier.with_telegram(&tg.bot_token, &tg.chat_id);
        }
        if let Some(ref webhook) = config.discord_webhook {
            notifier = notifier.with_discord(webhook);
        }
        
        Self {
            config,
            pid_file,
            auto_dream,
            notifier,
        }
    }

    /// 데몬 시작
    pub async fn start(&self) -> Result<()> {
        // PID 파일 생성
        self.write_pid()?;
        
        println!("🤖 KAIROS daemon started (PID: {})", process::id());
        println!("   Memory path: {:?}", self.config.memory_path);
        
        // 시작 알림
        self.notifier.send(&Notification::new(
            "KAIROS Started",
            "🤖 Background agent is now running"
        )).await?;
        
        // GitHub 이벤트 채널
        let (event_tx, mut event_rx) = mpsc::channel::<GitHubEvent>(100);
        let webhook_handler = WebhookHandler::new(event_tx);
        
        // 메인 루프
        loop {
            tokio::select! {
                // GitHub 이벤트 처리
                Some(event) = event_rx.recv() => {
                    let message = format_event(&event);
                    println!("📥 {}", message);
                    
                    // CI 실패 시 High priority 알림
                    let priority = match &event {
                        GitHubEvent::CheckRun { conclusion: Some(c), .. } if c == "failure" => Priority::High,
                        GitHubEvent::PullRequestReview { state, .. } if state == "changes_requested" => Priority::High,
                        _ => Priority::Normal,
                    };
                    
                    self.notifier.send(&Notification::new(
                        "GitHub Event",
                        &message
                    ).with_priority(priority)).await?;
                }
                
                // 1시간마다 Auto Dream 체크
                _ = sleep(Duration::from_secs(3600)) => {
                    if self.auto_dream.should_consolidate()? {
                        println!("🌙 Auto Dream triggered...");
                        match self.auto_dream.run_dream().await {
                            Ok(result) => {
                                println!("   Dream completed: {:?}", result);
                                self.notifier.send(&Notification::new(
                                    "Dream Completed",
                                    "🌙 Memory consolidation finished"
                                ).with_priority(Priority::Low)).await?;
                            }
                            Err(e) => eprintln!("   Dream error: {}", e),
                        }
                    }
                }
            }
        }
    }

    /// 데몬 중지
    pub fn stop(&self) -> Result<()> {
        if let Ok(pid_str) = fs::read_to_string(&self.pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                #[cfg(unix)]
                {
                    use std::process::Command;
                    let _ = Command::new("kill")
                        .args(["-TERM", &pid.to_string()])
                        .output();
                }
                println!("🛑 KAIROS daemon stopped (PID: {})", pid);
            }
        }
        let _ = fs::remove_file(&self.pid_file);
        Ok(())
    }

    /// 데몬 상태
    pub fn status(&self) -> Result<DaemonStatus> {
        if !self.pid_file.exists() {
            return Ok(DaemonStatus::Stopped);
        }
        
        let pid_str = fs::read_to_string(&self.pid_file)?;
        let pid: u32 = pid_str.trim().parse()?;
        
        #[cfg(unix)]
        {
            use std::process::Command;
            let output = Command::new("kill")
                .args(["-0", &pid.to_string()])
                .output()?;
            
            if output.status.success() {
                return Ok(DaemonStatus::Running { pid });
            }
        }
        
        Ok(DaemonStatus::Stopped)
    }

    /// PID 파일 작성
    fn write_pid(&self) -> Result<()> {
        if let Some(parent) = self.pid_file.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&self.pid_file)?;
        write!(file, "{}", process::id())?;
        Ok(())
    }
}

/// 데몬 상태
#[derive(Debug)]
pub enum DaemonStatus {
    Running { pid: u32 },
    Stopped,
}
