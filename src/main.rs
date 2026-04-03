mod api;
mod config;
mod engine;
mod kairos;
mod tools;

use clap::{Parser, Subcommand};
use kairos::{ConfigFile, KairosConfig, KairosDaemon, Notifier, start_server};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "sam")]
#[command(about = "🦊 Sam - Rust AI Agent with KAIROS", long_about = None)]
struct Cli {
    #[arg(trailing_var_arg = true)]
    task: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Chat,
    Run { task: String },
    Tools,
    #[command(subcommand)]
    Kairos(KairosCommands),
}

#[derive(Subcommand)]
enum KairosCommands {
    Start,
    Stop,
    Status,
    Dream,
    Log {
        #[arg(short, long, default_value = "1")]
        days: usize,
    },
    Serve {
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Show or initialize config
    Config {
        /// Initialize default config file
        #[arg(long)]
        init: bool,
        /// Show config file path
        #[arg(long)]
        path: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if !cli.task.is_empty() {
        let task = cli.task.join(" ");
        engine::run_task(&task).await?;
        return Ok(());
    }

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
        Some(Commands::Kairos(kairos_cmd)) => {
            handle_kairos(kairos_cmd).await?;
        }
    }

    Ok(())
}

async fn handle_kairos(cmd: KairosCommands) -> anyhow::Result<()> {
    match cmd {
        KairosCommands::Config { init, path } => {
            if path {
                println!("{:?}", ConfigFile::config_path());
            } else if init {
                ConfigFile::create_default()?;
            } else {
                // Show current config
                let config = ConfigFile::load()?;
                println!("{}", toml::to_string_pretty(&config)?);
            }
            return Ok(());
        }
        _ => {}
    }

    let config = KairosConfig::default();
    let daemon = KairosDaemon::new(config.clone());
    
    match cmd {
        KairosCommands::Start => {
            println!("🤖 Starting KAIROS daemon...");
            daemon.start().await?;
        }
        KairosCommands::Stop => {
            daemon.stop()?;
        }
        KairosCommands::Status => {
            match daemon.status()? {
                kairos::DaemonStatus::Running { pid } => {
                    println!("🤖 KAIROS is running (PID: {})", pid);
                }
                kairos::DaemonStatus::Stopped => {
                    println!("💤 KAIROS is not running");
                }
            }
        }
        KairosCommands::Dream => {
            println!("🌙 Running manual dream...");
            let auto_dream = kairos::AutoDream::new(config);
            match auto_dream.run_dream().await {
                Ok(result) => println!("Dream result: {:?}", result),
                Err(e) => eprintln!("Dream error: {}", e),
            }
        }
        KairosCommands::Log { days } => {
            let memdir = kairos::MemoryDir::new(&config.memory_path);
            let daily_log = kairos::DailyLog::new(memdir.logs_path());
            
            let logs = daily_log.list_recent(days)?;
            if logs.is_empty() {
                println!("📝 No logs found");
            } else {
                for log in logs {
                    if let Ok(content) = std::fs::read_to_string(&log) {
                        println!("{}", content);
                        println!("---");
                    }
                }
            }
        }
        KairosCommands::Serve { port } => {
            let mut config = config;
            if let Some(p) = port {
                config.server_port = p;
            }
            
            let (event_tx, mut event_rx) = mpsc::channel(100);
            let notifier = Arc::new(Notifier::new());
            
            let notifier_clone = notifier.clone();
            tokio::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    let message = kairos::format_event(&event);
                    println!("📥 {}", message);
                    let _ = notifier_clone.send(&kairos::Notification::new(
                        "GitHub Event",
                        &message
                    )).await;
                }
            });
            
            start_server(config, event_tx, notifier).await?;
        }
        KairosCommands::Config { .. } => unreachable!(),
    }
    
    Ok(())
}
