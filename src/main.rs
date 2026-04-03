mod api;
mod config;
mod engine;
mod kairos;
mod tools;

use clap::{Parser, Subcommand};
use kairos::{KairosConfig, KairosDaemon, Notifier, start_server};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "sam")]
#[command(about = "🦊 Sam - Rust AI Agent with KAIROS", long_about = None)]
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
    /// KAIROS autonomous agent mode
    #[command(subcommand)]
    Kairos(KairosCommands),
}

#[derive(Subcommand)]
enum KairosCommands {
    /// Start KAIROS daemon (foreground)
    Start,
    /// Stop KAIROS daemon
    Stop,
    /// Show KAIROS status
    Status,
    /// Run memory dream (manual)
    Dream,
    /// Show daily log
    Log {
        /// Number of days to show
        #[arg(short, long, default_value = "1")]
        days: usize,
    },
    /// Start HTTP server for webhooks
    Serve {
        /// Port number
        #[arg(short, long, default_value = "3847")]
        port: u16,
    },
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
        Some(Commands::Kairos(kairos_cmd)) => {
            handle_kairos(kairos_cmd).await?;
        }
    }

    Ok(())
}

async fn handle_kairos(cmd: KairosCommands) -> anyhow::Result<()> {
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
            config.server_port = port;
            
            let (event_tx, mut event_rx) = mpsc::channel(100);
            let notifier = Arc::new(Notifier::new());
            
            // 이벤트 처리 태스크
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
            
            // HTTP 서버 시작
            start_server(config, event_tx, notifier).await?;
        }
    }
    
    Ok(())
}
