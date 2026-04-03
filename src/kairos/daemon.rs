//! KAIROS 백그라운드 데몬
//! 
//! 터미널을 닫아도 계속 실행되는 백그라운드 에이전트

use super::{AutoDream, KairosConfig};
use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::time::Duration;
use tokio::time::sleep;

/// KAIROS 데몬
pub struct KairosDaemon {
    config: KairosConfig,
    pid_file: PathBuf,
    auto_dream: AutoDream,
}

impl KairosDaemon {
    pub fn new(config: KairosConfig) -> Self {
        let pid_file = config.memory_path.join(".kairos.pid");
        let auto_dream = AutoDream::new(config.clone());
        
        Self {
            config,
            pid_file,
            auto_dream,
        }
    }

    /// 데몬 시작
    pub async fn start(&self) -> Result<()> {
        // PID 파일 생성
        self.write_pid()?;
        
        println!("🤖 KAIROS daemon started (PID: {})", process::id());
        println!("   Memory path: {:?}", self.config.memory_path);
        
        // 메인 루프
        loop {
            // Auto Dream 체크
            if self.auto_dream.should_consolidate()? {
                println!("🌙 Auto Dream triggered...");
                match self.auto_dream.run_dream().await {
                    Ok(result) => println!("   Dream result: {:?}", result),
                    Err(e) => eprintln!("   Dream error: {}", e),
                }
            }
            
            // 1시간마다 체크
            sleep(Duration::from_secs(3600)).await;
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
