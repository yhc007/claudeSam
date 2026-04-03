//! 🤖 KAIROS - 자율 에이전트 모드
//! 
//! Claude Code의 KAIROS를 Rust로 구현
//! - 백그라운드 데몬 실행
//! - 자동 메모리 정리 (Auto Dream)
//! - 일별 로그 시스템

pub mod auto_dream;
pub mod consolidation;
pub mod daily_log;
pub mod daemon;
pub mod memdir;

pub use auto_dream::AutoDream;
pub use consolidation::ConsolidationLock;
pub use daily_log::DailyLog;
pub use daemon::KairosDaemon;
pub use memdir::MemoryDir;

/// KAIROS 설정
#[derive(Debug, Clone)]
pub struct KairosConfig {
    /// 메모리 정리 최소 간격 (시간)
    pub min_hours: u64,
    /// 메모리 정리 최소 세션 수
    pub min_sessions: u64,
    /// 메모리 디렉토리 경로
    pub memory_path: std::path::PathBuf,
}

impl Default for KairosConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            min_hours: 24,
            min_sessions: 5,
            memory_path: home.join(".claude-sam").join("memory"),
        }
    }
}
