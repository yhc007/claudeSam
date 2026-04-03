mod api;
mod config;
mod engine;
mod kairos;
mod tools;

use clap::{Parser, Subcommand};
use kairos::{
    ConfigFile, DaemonizeConfig, KairosConfig, KairosDaemon, MemoryBrain, 
    Notifier, PlanStatus, UltraPlan, daemonize, is_daemon_running, start_server, stop_daemon
};
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
    #[command(subcommand)]
    Brain(BrainCommands),
    /// Deep planning mode (ULTRAPLAN)
    Ultraplan {
        /// Planning prompt
        prompt: String,
        /// Timeout in minutes
        #[arg(short, long, default_value = "30")]
        timeout: u64,
        /// Skip approval prompt
        #[arg(long)]
        no_approve: bool,
    },
    /// Manage saved plans
    #[command(subcommand)]
    Plans(PlansCommands),
}

#[derive(Subcommand)]
enum KairosCommands {
    Start {
        #[arg(short, long)]
        daemon: bool,
    },
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
        #[arg(short, long)]
        daemon: bool,
    },
    Config {
        #[arg(long)]
        init: bool,
        #[arg(long)]
        path: bool,
    },
    Logs {
        #[arg(short, long)]
        follow: bool,
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },
}

#[derive(Subcommand)]
enum BrainCommands {
    Store {
        content: String,
        #[arg(short, long, default_value = "")]
        tags: String,
    },
    Recall {
        query: String,
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },
    Stats,
    Sleep,
}

#[derive(Subcommand)]
enum PlansCommands {
    /// List all plans
    List,
    /// Show a specific plan
    Show { id: String },
    /// Approve a plan
    Approve { id: String },
    /// Reject a plan
    Reject { id: String },
    /// Apply an approved plan
    Apply { id: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if !cli.task.is_empty() {
        let task = cli.task.join(" ");
        engine::run_task(&task).await?;
        return Ok(());
    }

    match cli.command {
        Some(Commands::Chat) | None => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .init();
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
        Some(Commands::Brain(brain_cmd)) => {
            handle_brain(brain_cmd)?;
        }
        Some(Commands::Ultraplan { prompt, timeout, no_approve }) => {
            handle_ultraplan(&prompt, timeout, no_approve).await?;
        }
        Some(Commands::Plans(plans_cmd)) => {
            handle_plans(plans_cmd).await?;
        }
    }

    Ok(())
}

async fn handle_ultraplan(prompt: &str, timeout: u64, no_approve: bool) -> anyhow::Result<()> {
    let config = KairosConfig::default();
    let ultraplan = UltraPlan::new(&config.memory_path);
    
    // Plan 생성
    let plan = ultraplan.run(prompt, timeout).await?;
    
    println!("\n📋 Plan saved: {}", plan.id);
    println!("   Location: ~/.claude-sam/memory/plans/{}.md", plan.id);
    
    if no_approve {
        println!("\n⏭️  Skipping approval (--no-approve)");
        return Ok(());
    }
    
    // 대화형 승인
    if ultraplan.interactive_approve(&plan)? {
        ultraplan.approve_plan(&plan.id)?;
        println!("\n✅ Plan approved!");
        
        print!("\n🚀 Apply now? (y/n): ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            let result = ultraplan.apply_plan(&plan.id).await?;
            println!("\n{}", result);
        }
    } else {
        ultraplan.reject_plan(&plan.id)?;
        println!("\n❌ Plan rejected");
    }
    
    Ok(())
}

async fn handle_plans(cmd: PlansCommands) -> anyhow::Result<()> {
    let config = KairosConfig::default();
    let ultraplan = UltraPlan::new(&config.memory_path);
    
    match cmd {
        PlansCommands::List => {
            let plans = ultraplan.list_plans()?;
            
            if plans.is_empty() {
                println!("📋 No plans found");
            } else {
                println!("📋 Plans:\n");
                for plan in plans {
                    let status_emoji = match plan.status {
                        PlanStatus::Pending => "⏳",
                        PlanStatus::Approved => "✅",
                        PlanStatus::Rejected => "❌",
                        PlanStatus::Applied => "🚀",
                    };
                    println!("{} {} - {} ({:?})", 
                        status_emoji, 
                        plan.id, 
                        if plan.prompt.len() > 50 { 
                            format!("{}...", &plan.prompt[..50]) 
                        } else { 
                            plan.prompt.clone() 
                        },
                        plan.status
                    );
                }
            }
        }
        PlansCommands::Show { id } => {
            let plan = ultraplan.load_plan(&id)?;
            println!("📋 Plan: {}\n", plan.id);
            println!("Status: {:?}", plan.status);
            println!("Created: {}", plan.created_at);
            println!("Prompt: {}\n", plan.prompt);
            println!("{}", "=".repeat(60));
            println!("{}", plan.content);
        }
        PlansCommands::Approve { id } => {
            ultraplan.approve_plan(&id)?;
            println!("✅ Plan {} approved", id);
        }
        PlansCommands::Reject { id } => {
            ultraplan.reject_plan(&id)?;
            println!("❌ Plan {} rejected", id);
        }
        PlansCommands::Apply { id } => {
            let result = ultraplan.apply_plan(&id).await?;
            println!("🚀 Plan applied:\n{}", result);
        }
    }
    
    Ok(())
}

fn handle_brain(cmd: BrainCommands) -> anyhow::Result<()> {
    let brain = MemoryBrain::new();
    
    match cmd {
        BrainCommands::Store { content, tags } => {
            let tags: Vec<&str> = if tags.is_empty() {
                vec![]
            } else {
                tags.split(',').collect()
            };
            
            println!("🧠 Storing memory...");
            let result = brain.store(&content, &tags)?;
            println!("{}", result);
        }
        BrainCommands::Recall { query, limit } => {
            println!("🧠 Recalling memories for: {}", query);
            let memories = brain.recall(&query, limit)?;
            
            if memories.is_empty() {
                println!("No memories found.");
            } else {
                for (i, mem) in memories.iter().enumerate() {
                    println!("{}. {}", i + 1, mem.content);
                }
            }
        }
        BrainCommands::Stats => {
            let stats = brain.stats()?;
            println!("{}", stats);
        }
        BrainCommands::Sleep => {
            println!("😴 Running memory consolidation...");
            let result = brain.sleep()?;
            println!("{}", result);
        }
    }
    
    Ok(())
}

async fn handle_kairos(cmd: KairosCommands) -> anyhow::Result<()> {
    let config = KairosConfig::default();
    let daemon_config = DaemonizeConfig::new(config.memory_path.clone());

    match cmd {
        KairosCommands::Config { init, path } => {
            if path {
                println!("{:?}", ConfigFile::config_path());
            } else if init {
                ConfigFile::create_default()?;
            } else {
                let cfg = ConfigFile::load()?;
                println!("{}", toml::to_string_pretty(&cfg)?);
            }
            return Ok(());
        }
        KairosCommands::Logs { follow, lines } => {
            let log_file = config.memory_path.join("kairos.log");
            if follow {
                let _ = std::process::Command::new("tail")
                    .args(["-f", "-n", &lines.to_string()])
                    .arg(&log_file)
                    .status();
            } else {
                let _ = std::process::Command::new("tail")
                    .args(["-n", &lines.to_string()])
                    .arg(&log_file)
                    .status();
            }
            return Ok(());
        }
        KairosCommands::Start { daemon: daemonize_flag } => {
            if daemonize_flag {
                println!("🤖 Starting KAIROS daemon in background...");
                daemonize(&daemon_config)?;
                println!("✅ KAIROS daemon started");
            }
            
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .init();
            
            let daemon = KairosDaemon::new(config);
            daemon.start().await?;
        }
        KairosCommands::Stop => {
            if stop_daemon(&daemon_config.pid_file)? {
                println!("🛑 KAIROS daemon stopped");
            } else {
                println!("💤 KAIROS daemon is not running");
            }
        }
        KairosCommands::Status => {
            if is_daemon_running(&daemon_config.pid_file) {
                let pid = std::fs::read_to_string(&daemon_config.pid_file).unwrap_or_default();
                println!("🤖 KAIROS is running (PID: {})", pid.trim());
            } else {
                println!("💤 KAIROS is not running");
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
        KairosCommands::Serve { port, daemon: daemonize_flag } => {
            let mut config = config;
            if let Some(p) = port {
                config.server_port = p;
            }
            
            if daemonize_flag {
                daemonize(&daemon_config)?;
            }
            
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .init();
            
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
    }
    
    Ok(())
}
