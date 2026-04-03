//! 메모리 디렉토리 관리
//! 
//! 구조:
//! ~/.claude-sam/memory/
//! ├── MEMORY.md         # 인덱스 (200줄/25KB 제한)
//! ├── user_role.md      # 사용자 정보
//! ├── project_*.md      # 프로젝트 컨텍스트
//! └── logs/
//!     └── YYYY/MM/YYYY-MM-DD.md  # 일별 로그

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub const ENTRYPOINT_NAME: &str = "MEMORY.md";
pub const MAX_ENTRYPOINT_LINES: usize = 200;
pub const MAX_ENTRYPOINT_BYTES: usize = 25_000;

/// 메모리 디렉토리 관리자
pub struct MemoryDir {
    base_path: PathBuf,
}

impl MemoryDir {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// 메모리 디렉토리 초기화
    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.base_path)?;
        fs::create_dir_all(self.logs_path())?;
        
        // MEMORY.md 없으면 생성
        let memory_file = self.memory_file();
        if !memory_file.exists() {
            fs::write(&memory_file, self.default_memory_content())?;
        }
        
        Ok(())
    }

    /// MEMORY.md 경로
    pub fn memory_file(&self) -> PathBuf {
        self.base_path.join(ENTRYPOINT_NAME)
    }

    /// logs 디렉토리 경로
    pub fn logs_path(&self) -> PathBuf {
        self.base_path.join("logs")
    }

    /// MEMORY.md 읽기 (truncate 적용)
    pub fn read_memory(&self) -> Result<String> {
        let content = fs::read_to_string(self.memory_file())?;
        Ok(self.truncate_content(&content))
    }

    /// MEMORY.md 쓰기
    pub fn write_memory(&self, content: &str) -> Result<()> {
        let truncated = self.truncate_content(content);
        fs::write(self.memory_file(), truncated)?;
        Ok(())
    }

    /// 컨텐츠 truncate (줄/바이트 제한)
    fn truncate_content(&self, content: &str) -> String {
        let trimmed = content.trim();
        let lines: Vec<&str> = trimmed.lines().collect();
        
        // 줄 수 제한
        let line_truncated: String = if lines.len() > MAX_ENTRYPOINT_LINES {
            lines[..MAX_ENTRYPOINT_LINES].join("\n")
        } else {
            trimmed.to_string()
        };
        
        // 바이트 제한
        if line_truncated.len() > MAX_ENTRYPOINT_BYTES {
            let truncated = &line_truncated[..MAX_ENTRYPOINT_BYTES];
            // 마지막 줄바꿈에서 자르기
            if let Some(pos) = truncated.rfind('\n') {
                truncated[..pos].to_string()
            } else {
                truncated.to_string()
            }
        } else {
            line_truncated
        }
    }

    /// 기본 MEMORY.md 내용
    fn default_memory_content(&self) -> String {
        r#"# 🧠 MEMORY.md

이 파일은 claudeSam의 장기 기억 인덱스입니다.

## 사용자 정보
- (아직 없음)

## 프로젝트 컨텍스트
- (아직 없음)

## 학습된 교훈
- (아직 없음)

---
*마지막 정리: 없음*
"#.to_string()
    }

    /// 메모리 파일 목록
    pub fn list_memory_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                files.push(path);
            }
        }
        
        Ok(files)
    }
}
