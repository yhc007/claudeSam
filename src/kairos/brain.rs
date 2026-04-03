//! memory-brain 연동
//! 
//! Paul의 시맨틱 메모리 시스템과 연동
//! - 작업 내용 자동 저장
//! - 관련 기억 recall
//! - Dream 시 활용

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// memory-brain CLI 경로
const BRAIN_PATH: &str = "/Volumes/T7/Work/memory-brain";

/// 메모리 항목
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Option<String>,
    pub content: String,
    pub tags: Vec<String>,
    pub score: Option<f32>,
}

/// memory-brain 클라이언트
pub struct MemoryBrain {
    brain_path: String,
}

impl MemoryBrain {
    pub fn new() -> Self {
        Self {
            brain_path: BRAIN_PATH.to_string(),
        }
    }

    /// 메모리 저장
    pub fn store(&self, content: &str, tags: &[&str]) -> Result<String> {
        let tags_str = tags.join(",");
        
        let output = Command::new("cargo")
            .current_dir(&self.brain_path)
            .args(["run", "--release", "-q", "--", "store", content, "--tags", &tags_str])
            .env("EMBEDDING_SERVER_URL", "http://localhost:3201")
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // ID 추출 (출력에서 파싱)
        let id = stdout
            .lines()
            .find(|l| l.contains("ID:") || l.contains("Stored"))
            .map(|l| l.to_string())
            .unwrap_or_else(|| "stored".to_string());
        
        Ok(id)
    }

    /// 메모리 검색 (recall)
    pub fn recall(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let output = Command::new("cargo")
            .current_dir(&self.brain_path)
            .args([
                "run", "--release", "-q", "--",
                "recall", query,
                "--limit", &limit.to_string(),
                "--json"
            ])
            .env("EMBEDDING_SERVER_URL", "http://localhost:3201")
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // JSON 파싱 시도, 실패하면 텍스트로 파싱
        if let Ok(memories) = serde_json::from_str::<Vec<Memory>>(&stdout) {
            Ok(memories)
        } else {
            // 텍스트 출력 파싱
            let memories: Vec<Memory> = stdout
                .lines()
                .filter(|l| !l.is_empty() && !l.starts_with("🧪"))
                .map(|l| Memory {
                    id: None,
                    content: l.to_string(),
                    tags: vec![],
                    score: None,
                })
                .collect();
            Ok(memories)
        }
    }

    /// 시맨틱 검색
    pub fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let output = Command::new("cargo")
            .current_dir(&self.brain_path)
            .args([
                "run", "--release", "-q", "--",
                "search", query,
                "--limit", &limit.to_string()
            ])
            .env("EMBEDDING_SERVER_URL", "http://localhost:3201")
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        let memories: Vec<Memory> = stdout
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with("🧪"))
            .map(|l| Memory {
                id: None,
                content: l.to_string(),
                tags: vec![],
                score: None,
            })
            .collect();
        
        Ok(memories)
    }

    /// 통계 조회
    pub fn stats(&self) -> Result<String> {
        let output = Command::new("cargo")
            .current_dir(&self.brain_path)
            .args(["run", "--release", "-q", "--", "stats"])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 수면 (메모리 정리)
    pub fn sleep(&self) -> Result<String> {
        let output = Command::new("cargo")
            .current_dir(&self.brain_path)
            .args(["run", "--release", "-q", "--", "sleep"])
            .env("EMBEDDING_SERVER_URL", "http://localhost:3201")
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 헬스 체크
    pub fn is_available(&self) -> bool {
        Command::new("cargo")
            .current_dir(&self.brain_path)
            .args(["run", "--release", "-q", "--", "stats"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for MemoryBrain {
    fn default() -> Self {
        Self::new()
    }
}
