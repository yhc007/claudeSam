use crate::api::{AnthropicClient, ContentBlock, Message, MessageContent, Tool as ApiTool};
use crate::config::Config;
use crate::tools::{get_all_tools, Tool};
use anyhow::Result;
use serde_json::json;
use std::io::{self, Write};

pub async fn run_chat() -> Result<()> {
    let config = Config::load()?;
    let client = AnthropicClient::new(config.api_key);
    let tools = get_all_tools();
    let mut messages: Vec<Message> = Vec::new();

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

        let response = process_with_tools(&client, &tools, &mut messages).await?;
        println!("\n🦊 {}\n", response);
    }

    Ok(())
}

pub async fn run_task(task: &str) -> Result<()> {
    let config = Config::load()?;
    let client = AnthropicClient::new(config.api_key);
    let tools = get_all_tools();
    let mut messages = vec![Message {
        role: "user".to_string(),
        content: MessageContent::Text(task.to_string()),
    }];

    let response = process_with_tools(&client, &tools, &mut messages).await?;
    println!("\n🦊 {}", response);

    Ok(())
}

async fn process_with_tools(
    client: &AnthropicClient,
    tools: &[Box<dyn Tool>],
    messages: &mut Vec<Message>,
) -> Result<String> {
    let api_tools: Vec<ApiTool> = tools
        .iter()
        .map(|t| ApiTool {
            name: t.name().to_string(),
            description: t.description().to_string(),
            input_schema: t.input_schema(),
        })
        .collect();

    loop {
        let response = client
            .send_message(messages.clone(), Some(api_tools.clone()))
            .await?;

        let mut tool_uses = Vec::new();
        let mut text_response = String::new();

        for block in &response.content {
            match block.block_type.as_str() {
                "text" => {
                    if let Some(text) = &block.text {
                        text_response.push_str(text);
                    }
                }
                "tool_use" => {
                    tool_uses.push(block.clone());
                }
                _ => {}
            }
        }

        if tool_uses.is_empty() {
            return Ok(text_response);
        }

        // Add assistant message with tool calls
        messages.push(Message {
            role: "assistant".to_string(),
            content: MessageContent::Blocks(response.content),
        });

        // Execute tools and collect results
        let mut tool_results = Vec::new();
        for tool_use in tool_uses {
            let tool_name = tool_use.name.as_deref().unwrap_or("");
            let tool_id = tool_use.id.as_deref().unwrap_or("");
            let input = tool_use.input.clone().unwrap_or(json!({}));

            println!("⚙️  Running tool: {} ...", tool_name);

            let result = if let Some(tool) = tools.iter().find(|t| t.name() == tool_name) {
                tool.execute(input).await.unwrap_or_else(|e| e.to_string())
            } else {
                format!("Unknown tool: {}", tool_name)
            };

            tool_results.push(ContentBlock {
                block_type: "tool_result".to_string(),
                text: Some(result),
                id: Some(tool_id.to_string()),
                name: None,
                input: None,
            });
        }

        messages.push(Message {
            role: "user".to_string(),
            content: MessageContent::Blocks(tool_results),
        });
    }
}
