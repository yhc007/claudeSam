pub mod bash;
pub mod file;
pub mod grep;

use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, input: Value) -> anyhow::Result<String>;
}

pub fn list_tools() {
    println!("Available tools:");
    println!("  bash  - Execute shell commands");
    println!("  file  - Read/write files");
    println!("  grep  - Search in files");
}

pub fn get_all_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(bash::BashTool),
        Box::new(file::FileTool),
        Box::new(grep::GrepTool),
    ]
}
