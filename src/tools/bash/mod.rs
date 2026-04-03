use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Command;

pub struct BashTool;

const BLOCKED_COMMANDS: &[&str] = &["rm -rf /", "mkfs", "dd if=", ":(){:|:&};:"];

#[async_trait]
impl super::Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute shell commands. Use for file operations, running programs, etc."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, input: Value) -> anyhow::Result<String> {
        let command = input["command"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing command"))?;

        // Security check
        for blocked in BLOCKED_COMMANDS {
            if command.contains(blocked) {
                return Ok(format!("🚫 Blocked dangerous command: {}", command));
            }
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(stdout.to_string())
        } else {
            Ok(format!("Error (exit {}): {}\n{}", 
                output.status.code().unwrap_or(-1), 
                stdout, 
                stderr))
        }
    }
}
