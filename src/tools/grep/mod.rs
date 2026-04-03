use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Command;

pub struct GrepTool;

#[async_trait]
impl super::Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search for patterns in files using ripgrep"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Search pattern"
                },
                "path": {
                    "type": "string",
                    "description": "Directory or file to search"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, input: Value) -> anyhow::Result<String> {
        let pattern = input["pattern"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing pattern"))?;
        let path = input["path"].as_str().unwrap_or(".");

        let output = Command::new("rg")
            .args(["--color=never", "-n", pattern, path])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.is_empty() {
            Ok("No matches found".to_string())
        } else {
            Ok(stdout.to_string())
        }
    }
}
