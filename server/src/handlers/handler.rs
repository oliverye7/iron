use std::fmt::Result;
use std::sync::Arc;

use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;

use crate::db::db::dummy_db_function;
use crate::handlers::chat::handle_openai_call;
use crate::handlers::cli::handle_cli_command;
use crate::state::app_state::{ChatState, CliCommandType, ContextMessage, MessageType};

use super::cli;

#[derive(Debug, Serialize, Deserialize)]
pub struct AssistantResponse {
    output: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CliResponse {
    output: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliCommand {
    command_type: CliCommandType,
    command: String,
}

pub async fn handle_chat_action(typed_msg: &ContextMessage, chat_state: Arc<ChatState>) -> Result {
    // does it make sense to create a chat_action struct?
    // if so:
    // chat actions consist of the below (the field names can be improved):
    // trigger_message: input either from user or an LLM call tasking a next chat action to begin (e.g. "please explore the codebase")
    // llm_response: llm's initial analysis response to the trigger message (e.g. "let me begin by running ls")
    // cli_command: llm's proposed cli_command (e.g. "ls")
    // cli_output: output of running the cli_command
    // llm_summary: llm's summary of output in the context of the global context (e.g. "it looks like there are files xyz here")
    //
    // this function can either be triggered by a user's request for an action, or an LLM's continuation
    // TODO: i think the cloning here is unnecessary, right? the function call can just access the memory instead of owning it
    let llm_response = openai_message(typed_msg.clone(), chat_state.clone()).await;
    // TODO: send the llm response back to the websocket server's write stream to the FE.
    // update the db by calling dummy_db_function for now; or should the functions the handler hands off to take care of the db updates?
    dummy_db_function();
    // decide if there needs to be any CLI action here
    let command = extract_command_from_frontend_message(typed_msg.content.clone());
    if let Some(command) = command {
        let cli_response = cli_command(command, chat_state.clone()).await;
        if cli_response.status == "success" {
            let cli_message = ContextMessage {
                message_type: MessageType::CliOutput,
                content: cli_response.output,
                timestamp: Some(chrono::Utc::now()),
            };
            // TODO: the cli response should automatically be streamed, since the CLI has a websocket connection
            // open with the websocket server.
            let llm_response = openai_message(cli_message, chat_state).await;
            // TODO: send the llm response back to the websocket server's write stream to the FE.
        }
        return Ok(());
    } else {
        // return, the chat action has finished
        return Ok(());
    }
}

async fn openai_message(new_message: ContextMessage, state: Arc<ChatState>) -> AssistantResponse {
    match handle_openai_call(&new_message, &state).await {
        Ok(output) => AssistantResponse {
            output,
            status: "success".to_string(),
        },
        Err(e) => AssistantResponse {
            output: e.to_string(),
            // TODO: make verbose?
            status: "error".to_string(),
        },
    }
}

async fn cli_command(command: CliCommand, state: Arc<ChatState>) -> CliResponse {
    match handle_cli_command(
        command.command.clone(),
        command.command_type.clone(),
        &state,
    )
    .await
    {
        Ok(output) => CliResponse {
            output,
            status: "success".to_string(),
        },
        Err(e) => CliResponse {
            output: e.to_string(),
            // TODO: make verbose?
            status: "error".to_string(),
        },
    }
}

fn extract_command_from_frontend_message(command: String) -> Option<CliCommand> {
    // this should be simple, the LLM should structure its output such that it specifies whether the cmd is READONLY or WRITE/EXECUTE
    let cmd = CliCommand {
        command_type: CliCommandType::WriteExecuteCliCommand,
        command,
    };
    return Some(cmd);
}
