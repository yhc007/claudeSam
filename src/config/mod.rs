use anyhow::Result;

pub struct Config {
    pub cli_path: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let cli_path = std::env::var("CLAUDE_CLI_PATH")
            .unwrap_or_else(|_| "claude".to_string());
        
        Ok(Self { cli_path })
    }
}
