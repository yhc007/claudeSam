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
    #[command(subcommand)]
    command: Commands,
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

    match cli.command {
        Commands::Chat => {
            println!("🦊 Sam is ready! Type your message:");
            engine::run_chat().await?;
        }
        Commands::Run { task } => {
            println!("🦊 Running task: {}", task);
            engine::run_task(&task).await?;
        }
        Commands::Tools => {
            tools::list_tools();
        }
    }

    Ok(())
}
