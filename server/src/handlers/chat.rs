// llm chat handlers
use dotenv::dotenv;
use reqwest::Client;
use serde_json::json;
use std::env;
use std::fs;

use crate::state::app_state::ChatState;
use crate::state::app_state::ContextMessage;
use crate::state::app_state::MessageType;

pub(crate) async fn handle_openai_call(
    new_message: &ContextMessage,
    state: &ChatState,
) -> Result<String, Box<dyn std::error::Error + 'static>> {
    // TODO: make this function return a stream, so the frontend can consume the stream via the websocket
    // TODO: however, also need to make sure that the function calling this receives all the output at once,
    // since it has to update context in discrete message actions
    dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY").expect("Missing OPENAI_API_KEY env variable");
    let request_url = "https://api.openai.com/v1/chat/completions";
    // TODO: is there a better way to do this?
    let system_prompt =
        fs::read_to_string("system_prompt.txt").expect("Failed to read system_prompt.txt");
    state.add_message_to_state(
        new_message.message_type.clone(),
        new_message.content.clone(),
    )?;

    let client = Client::new();
    let response = client
        .post(request_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
                "model": "gpt-4o-mini",
                "store": true,
                "messages": [
                    {
                        "role": "developer",
                        "content": system_prompt
                    },
                    {
                        "role": "user",
                        "content": new_message.content
                    },
                ]
        }))
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;
    // TODO: probably good to define an openai response struct so it can get typed instantly and we can refer
    // to local code typing for documentation instead of reading oai documentation to know how to extract what
    let completion = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response from OpenAI")
        .to_string();

    state.add_message_to_state(MessageType::AssistantResponse, completion.clone())?;

    // TODO: handle command extraction for autorun
    //if completion.contains("COMMAND (READ-ONLY):") {
    //    // Extract the command after "COMMAND (READ-ONLY):"
    //    if let Some(command) = completion.split("COMMAND (READ-ONLY):").nth(1) {
    //        let extracted_command = command.trim().lines().next().unwrap_or("").to_string();
    //        let cli_command = CliCommand {
    //            command: String::from(extracted_command),
    //        };
    //        cli_command(cli_command, prev_context);
    //    }
    //}
    return Ok(completion);
}
