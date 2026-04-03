//! ULTRAPLAN - 30분 딥 플래닝 기능
//! 
//! Claude Code의 ULTRAPLAN을 Rust로 구현
//! - 장시간 심층 기획 (최대 30분)
//! - Plan 파일 저장 및 관리
//! - 승인 후 적용

use anyhow::Result;
use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Plan 상태
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanStatus {
    Pending,    // 생성됨, 미승인
    Approved,   // 승인됨
    Rejected,   // 거절됨
    Applied,    // 적용됨
}

/// Plan 메타데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub prompt: String,
    pub content: String,
    pub status: PlanStatus,
    pub created_at: String,
    pub updated_at: String,
    pub timeout_mins: u64,
}

impl Plan {
    pub fn new(prompt: &str, content: &str, timeout_mins: u64) -> Self {
        let now = Utc::now().to_rfc3339();
        let id = format!("plan_{}", Local::now().format("%Y%m%d_%H%M%S"));
        
        Self {
            id,
            prompt: prompt.to_string(),
            content: content.to_string(),
            status: PlanStatus::Pending,
            created_at: now.clone(),
            updated_at: now,
            timeout_mins,
        }
    }
}

/// ULTRAPLAN 관리자
pub struct UltraPlan {
    plans_dir: PathBuf,
    claude_cli: String,
}

impl UltraPlan {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            plans_dir: base_path.as_ref().join("plans"),
            claude_cli: "claude".to_string(),
        }
    }

    /// Plans 디렉토리 초기화
    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.plans_dir)?;
        Ok(())
    }

    /// ULTRAPLAN 실행
    pub async fn run(&self, prompt: &str, timeout_mins: u64) -> Result<Plan> {
        self.init()?;

        println!("🧠 ULTRAPLAN starting...");
        println!("   Prompt: {}", prompt);
        println!("   Timeout: {} minutes", timeout_mins);
        println!();

        // Claude CLI로 장시간 플래닝 실행
        let ultraplan_prompt = format!(
            r#"You are in ULTRAPLAN mode - a deep planning session.

Task: {}

Please provide a comprehensive, detailed plan. Take your time to think through:
1. Goals and objectives
2. Step-by-step implementation plan
3. Potential challenges and solutions
4. Resource requirements
5. Timeline estimates
6. Success criteria

Be thorough and specific. This plan will be reviewed before execution."#,
            prompt
        );

        println!("⏳ Running deep planning (this may take a while)...");

        let output = Command::new(&self.claude_cli)
            .args([
                "--print",
                &ultraplan_prompt,
            ])
            .output()?;

        let content = String::from_utf8_lossy(&output.stdout).to_string();

        if content.trim().is_empty() {
            anyhow::bail!("No plan generated");
        }

        // Plan 생성 및 저장
        let plan = Plan::new(prompt, &content, timeout_mins);
        self.save_plan(&plan)?;

        println!("\n✅ Plan generated: {}", plan.id);

        Ok(plan)
    }

    /// Plan 저장
    fn save_plan(&self, plan: &Plan) -> Result<()> {
        let path = self.plans_dir.join(format!("{}.json", plan.id));
        let json = serde_json::to_string_pretty(plan)?;
        fs::write(&path, json)?;
        
        // Markdown 버전도 저장
        let md_path = self.plans_dir.join(format!("{}.md", plan.id));
        let md_content = format!(
            "# {}\n\n**Prompt:** {}\n\n**Status:** {:?}\n\n**Created:** {}\n\n---\n\n{}",
            plan.id, plan.prompt, plan.status, plan.created_at, plan.content
        );
        fs::write(&md_path, md_content)?;
        
        Ok(())
    }

    /// Plan 로드
    pub fn load_plan(&self, id: &str) -> Result<Plan> {
        let path = self.plans_dir.join(format!("{}.json", id));
        let content = fs::read_to_string(&path)?;
        let plan: Plan = serde_json::from_str(&content)?;
        Ok(plan)
    }

    /// 모든 Plan 목록
    pub fn list_plans(&self) -> Result<Vec<Plan>> {
        self.init()?;
        
        let mut plans = Vec::new();
        
        for entry in fs::read_dir(&self.plans_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(plan) = serde_json::from_str::<Plan>(&content) {
                        plans.push(plan);
                    }
                }
            }
        }
        
        // 최신순 정렬
        plans.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(plans)
    }

    /// Plan 승인
    pub fn approve_plan(&self, id: &str) -> Result<Plan> {
        let mut plan = self.load_plan(id)?;
        plan.status = PlanStatus::Approved;
        plan.updated_at = Utc::now().to_rfc3339();
        self.save_plan(&plan)?;
        Ok(plan)
    }

    /// Plan 거절
    pub fn reject_plan(&self, id: &str) -> Result<Plan> {
        let mut plan = self.load_plan(id)?;
        plan.status = PlanStatus::Rejected;
        plan.updated_at = Utc::now().to_rfc3339();
        self.save_plan(&plan)?;
        Ok(plan)
    }

    /// Plan 적용 (승인된 Plan만)
    pub async fn apply_plan(&self, id: &str) -> Result<String> {
        let mut plan = self.load_plan(id)?;
        
        if plan.status != PlanStatus::Approved {
            anyhow::bail!("Plan must be approved before applying. Current status: {:?}", plan.status);
        }

        println!("🚀 Applying plan: {}", id);

        // Plan 내용을 Claude에게 실행 요청
        let apply_prompt = format!(
            r#"Execute the following approved plan:

{}

Please implement this plan step by step. Report progress as you go."#,
            plan.content
        );

        let output = Command::new(&self.claude_cli)
            .args(["--print", &apply_prompt])
            .output()?;

        let result = String::from_utf8_lossy(&output.stdout).to_string();

        // 상태 업데이트
        plan.status = PlanStatus::Applied;
        plan.updated_at = Utc::now().to_rfc3339();
        self.save_plan(&plan)?;

        Ok(result)
    }

    /// 대화형 승인 프롬프트
    pub fn interactive_approve(&self, plan: &Plan) -> Result<bool> {
        println!("\n{}", "=".repeat(60));
        println!("📋 PLAN: {}", plan.id);
        println!("{}", "=".repeat(60));
        println!("\n{}\n", plan.content);
        println!("{}", "=".repeat(60));
        
        print!("\n승인하시겠습니까? (y/n/e[dit]): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => Ok(false),
        }
    }
}
