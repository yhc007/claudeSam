//! 분산 락 시스템
//! 
//! 여러 프로세스가 동시에 메모리 정리하는 것을 방지
//! PID 기반 락 + mtime으로 lastConsolidatedAt 추적

use anyhow::Result;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const LOCK_FILE: &str = ".consolidate-lock";
const HOLDER_STALE_MS: u64 = 60 * 60 * 1000; // 1시간

/// 분산 락 관리자
pub struct ConsolidationLock {
    lock_path: PathBuf,
}

impl ConsolidationLock {
    pub fn new(memory_path: impl AsRef<Path>) -> Self {
        Self {
            lock_path: memory_path.as_ref().join(LOCK_FILE),
        }
    }

    /// 마지막 정리 시간 (mtime)
    pub fn last_consolidated_at(&self) -> Result<u64> {
        match fs::metadata(&self.lock_path) {
            Ok(meta) => {
                let mtime = meta.modified()?;
                let duration = mtime.duration_since(UNIX_EPOCH)?;
                Ok(duration.as_millis() as u64)
            }
            Err(_) => Ok(0),
        }
    }

    /// 락 획득 시도
    /// 성공하면 Some(이전 mtime), 실패하면 None
    pub fn try_acquire(&self) -> Result<Option<u64>> {
        let prior_mtime = self.last_consolidated_at()?;
        
        // 기존 락 확인
        if let Ok(mut file) = File::open(&self.lock_path) {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_millis() as u64;
            
            // 최근 락이면 확인
            if now - prior_mtime < HOLDER_STALE_MS {
                if let Ok(pid) = content.trim().parse::<u32>() {
                    if Self::is_process_running(pid) {
                        // 다른 프로세스가 실행 중
                        return Ok(None);
                    }
                }
            }
        }
        
        // 락 획득
        if let Some(parent) = self.lock_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = File::create(&self.lock_path)?;
        write!(file, "{}", process::id())?;
        
        // 락 검증 (레이스 컨디션 방지)
        let verify = fs::read_to_string(&self.lock_path)?;
        if verify.trim().parse::<u32>().ok() != Some(process::id()) {
            return Ok(None);
        }
        
        Ok(Some(prior_mtime))
    }

    /// 락 해제 (롤백)
    pub fn rollback(&self, prior_mtime: u64) -> Result<()> {
        if prior_mtime == 0 {
            // 이전에 락 파일 없었음 - 삭제
            let _ = fs::remove_file(&self.lock_path);
        } else {
            // mtime 복원
            fs::write(&self.lock_path, "")?;
            let time = UNIX_EPOCH + Duration::from_millis(prior_mtime);
            filetime::set_file_mtime(&self.lock_path, filetime::FileTime::from_system_time(time))?;
        }
        Ok(())
    }

    /// 정리 완료 기록
    pub fn record_consolidation(&self) -> Result<()> {
        if let Some(parent) = self.lock_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&self.lock_path)?;
        write!(file, "{}", process::id())?;
        Ok(())
    }

    /// 프로세스 실행 중인지 확인
    fn is_process_running(pid: u32) -> bool {
        #[cfg(unix)]
        {
            use std::process::Command;
            Command::new("kill")
                .args(["-0", &pid.to_string()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(windows)]
        {
            // Windows에서는 tasklist 사용
            false
        }
    }
}
