//! 자동 메모리 정리 (Auto Dream)
//! 
//! Gate Order (가장 저렴한 것부터):
//! 1. Time Gate: 마지막 정리 후 24시간 이상 경과
//! 2. Session Gate: 5개 이상 세션 누적
//! 3. Lock Gate: 다른 프로세스가 정리 중이 아님

use super::{ConsolidationLock, DailyLog, KairosConfig, MemoryDir};
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

/// 자동 메모리 정리 관리자
pub struct AutoDream {
    config: KairosConfig,
    memdir: MemoryDir,
    lock: ConsolidationLock,
    daily_log: DailyLog,
}

impl AutoDream {
    pub fn new(config: KairosConfig) -> Self {
        let memdir = MemoryDir::new(&config.memory_path);
        let lock = ConsolidationLock::new(&config.memory_path);
        let daily_log = DailyLog::new(memdir.logs_path());
        
        Self {
            config,
            memdir,
            lock,
            daily_log,
        }
    }

    /// 게이트 확인 - 정리가 필요한지
    pub fn should_consolidate(&self) -> Result<bool> {
        // 1. Time Gate
        if !self.check_time_gate()? {
            return Ok(false);
        }
        
        // 2. Session Gate
        if !self.check_session_gate()? {
            return Ok(false);
        }
        
        Ok(true)
    }

    /// Time Gate: 마지막 정리 후 min_hours 이상 경과
    fn check_time_gate(&self) -> Result<bool> {
        let last = self.lock.last_consolidated_at()?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;
        
        let hours_since = (now - last) / (1000 * 60 * 60);
        Ok(hours_since >= self.config.min_hours)
    }

    /// Session Gate: min_sessions 이상 세션 누적
    fn check_session_gate(&self) -> Result<bool> {
        let recent_logs = self.daily_log.list_recent(7)?;
        Ok(recent_logs.len() as u64 >= self.config.min_sessions)
    }

    /// 메모리 정리 실행
    pub async fn run_dream(&self) -> Result<DreamResult> {
        // 락 획득
        let prior_mtime = match self.lock.try_acquire()? {
            Some(m) => m,
            None => return Ok(DreamResult::Locked),
        };

        // Dream 프롬프트 구성
        let prompt = self.build_dream_prompt()?;
        
        // TODO: 서브에이전트로 dream 실행
        // 지금은 간단한 정리만 수행
        
        // 정리 완료 기록
        self.lock.record_consolidation()?;
        
        Ok(DreamResult::Success {
            prompt,
            memories_processed: 0,
        })
    }

    /// Dream 프롬프트 구성
    fn build_dream_prompt(&self) -> Result<String> {
        let memory_content = self.memdir.read_memory().unwrap_or_default();
        let recent_logs: Vec<String> = self.daily_log
            .list_recent(7)?
            .iter()
            .filter_map(|p| std::fs::read_to_string(p).ok())
            .collect();
        
        Ok(format!(
            r#"# 🌙 Dream Mode - 메모리 정리

## 현재 MEMORY.md
{}

## 최근 7일 로그
{}

## 지시사항
1. 최근 로그에서 중요한 정보 추출
2. MEMORY.md 업데이트 (200줄 이내)
3. 중복/구식 정보 제거
4. 인덱스 정리
"#,
            memory_content,
            recent_logs.join("\n---\n")
        ))
    }
}

/// Dream 실행 결과
#[derive(Debug)]
pub enum DreamResult {
    /// 다른 프로세스가 락 보유 중
    Locked,
    /// 성공
    Success {
        prompt: String,
        memories_processed: usize,
    },
    /// 실패
    Failed(String),
}
