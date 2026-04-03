//! 일별 로그 관리
//! 
//! KAIROS 모드에서는 MEMORY.md 직접 편집 대신
//! 일별 로그 파일에 append-only로 기록

use anyhow::Result;
use chrono::{Datelike, Local};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// 일별 로그 관리자
pub struct DailyLog {
    logs_path: PathBuf,
}

impl DailyLog {
    pub fn new(logs_path: impl AsRef<Path>) -> Self {
        Self {
            logs_path: logs_path.as_ref().to_path_buf(),
        }
    }

    /// 오늘 로그 파일 경로
    pub fn today_log_path(&self) -> PathBuf {
        let now = Local::now();
        let year = now.year();
        let month = now.month();
        let day = now.day();
        
        self.logs_path
            .join(format!("{:04}", year))
            .join(format!("{:02}", month))
            .join(format!("{:04}-{:02}-{:02}.md", year, month, day))
    }

    /// 로그 추가 (append-only)
    pub fn append(&self, content: &str) -> Result<()> {
        let log_path = self.today_log_path();
        
        // 디렉토리 생성
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 파일이 없으면 헤더 추가
        let is_new = !log_path.exists();
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        
        if is_new {
            let now = Local::now();
            writeln!(file, "# 📅 {}", now.format("%Y-%m-%d"))?;
            writeln!(file)?;
        }
        
        // 타임스탬프와 함께 기록
        let now = Local::now();
        writeln!(file, "## {}", now.format("%H:%M:%S"))?;
        writeln!(file, "{}", content)?;
        writeln!(file)?;
        
        Ok(())
    }

    /// 오늘 로그 읽기
    pub fn read_today(&self) -> Result<String> {
        let log_path = self.today_log_path();
        if log_path.exists() {
            Ok(fs::read_to_string(log_path)?)
        } else {
            Ok(String::new())
        }
    }

    /// 특정 날짜 로그 읽기
    pub fn read_date(&self, year: i32, month: u32, day: u32) -> Result<String> {
        let log_path = self.logs_path
            .join(format!("{:04}", year))
            .join(format!("{:02}", month))
            .join(format!("{:04}-{:02}-{:02}.md", year, month, day));
        
        if log_path.exists() {
            Ok(fs::read_to_string(log_path)?)
        } else {
            Ok(String::new())
        }
    }

    /// 최근 N일 로그 목록
    pub fn list_recent(&self, days: usize) -> Result<Vec<PathBuf>> {
        let mut logs = Vec::new();
        let now = Local::now();
        
        for i in 0..days {
            let date = now - chrono::Duration::days(i as i64);
            let log_path = self.logs_path
                .join(format!("{:04}", date.year()))
                .join(format!("{:02}", date.month()))
                .join(format!("{:04}-{:02}-{:02}.md", date.year(), date.month(), date.day()));
            
            if log_path.exists() {
                logs.push(log_path);
            }
        }
        
        Ok(logs)
    }
}
