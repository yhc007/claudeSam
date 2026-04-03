mod api;
mod config;
mod engine;
mod tools;

use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "sam")]
#[command(about = "🦊 Sam - Rust AI Agent", long_about = None)]
struct Cli {
    /// Task to run (shortcut for 'sam run "task"')
    #[arg(trailing_var_arg = true)]
    task: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive chat
    Chat,
    /// Run a single task
    Run {
        /// Task description
        task: String,
    },
    /// List available tools
    Tools,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    // 인자가 있으면 바로 run
    if !cli.task.is_empty() {
        let task = cli.task.join(" ");
        engine::run_task(&task).await?;
        return Ok(());
    }

    // 서브커맨드 처리
    match cli.command {
        Some(Commands::Chat) | None => {
            engine::run_chat().await?;
        }
        Some(Commands::Run { task }) => {
            engine::run_task(&task).await?;
        }
        Some(Commands::Tools) => {
            tools::list_tools();
        }
    }

    Ok(())
}
