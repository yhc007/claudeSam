use crate::api::{ClaudeClient, ContentBlock, Message, MessageContent};
use anyhow::Result;
use std::io::{self, Write};

pub async fn run_chat() -> Result<()> {
    let client = ClaudeClient::new();
    let mut messages: Vec<Message> = Vec::new();

    println!("🦊 claudeSam ready! (Powered by Claude Code CLI)");
    println!("   Type 'exit' to quit.\n");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() || input == "exit" || input == "quit" {
            println!("🦊 Bye!");
            break;
        }

        messages.push(Message {
            role: "user".to_string(),
            content: MessageContent::Text(input.to_string()),
        });

        print!("🦊 Thinking...");
        io::stdout().flush()?;

        let response = client.send_message(messages.clone(), None).await?;
        
        // 줄 지우기
        print!("\r                \r");

        let response_text = response
            .content
            .iter()
            .filter_map(|b| b.text.as_ref())
            .cloned().collect::<Vec<_>>()
            .join("\n");

        println!("🦊 {}\n", response_text);

        messages.push(Message {
            role: "assistant".to_string(),
            content: MessageContent::Text(response_text),
        });
    }

    Ok(())
}

pub async fn run_task(task: &str) -> Result<()> {
    let client = ClaudeClient::new();
    let messages = vec![Message {
        role: "user".to_string(),
        content: MessageContent::Text(task.to_string()),
    }];

    println!("🦊 Running task with Claude Code CLI...\n");

    let response = client.send_message(messages, None).await?;
    
    let response_text = response
        .content
        .iter()
        .filter_map(|b| b.text.as_ref())
        .cloned().collect::<Vec<_>>()
        .join("\n");

    println!("{}", response_text);

    Ok(())
}
