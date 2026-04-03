use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;

pub struct FileTool;

#[async_trait]
impl super::Tool for FileTool {
    fn name(&self) -> &str {
        "file"
    }

    fn description(&self) -> &str {
        "Read or write files. Actions: read, write"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["read", "write"],
                    "description": "read or write"
                },
                "path": {
                    "type": "string",
                    "description": "File path"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write (for write action)"
                }
            },
            "required": ["action", "path"]
        })
    }

    async fn execute(&self, input: Value) -> anyhow::Result<String> {
        let action = input["action"].as_str().unwrap_or("read");
        let path = input["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?;

        match action {
            "read" => {
                let content = fs::read_to_string(path)?;
                Ok(content)
            }
            "write" => {
                let content = input["content"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing content"))?;
                fs::write(path, content)?;
                Ok(format!("✅ Wrote {} bytes to {}", content.len(), path))
            }
            _ => Ok(format!("Unknown action: {}", action)),
        }
    }
}
