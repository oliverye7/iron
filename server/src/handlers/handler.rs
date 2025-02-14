use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use std::result::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use tokio::net::TcpStream;

use serde::Deserialize;
use serde::Serialize;
use tokio_tungstenite::WebSocketStream;

use crate::db::db::dummy_db_function;
use crate::handlers::chat::handle_openai_call;
use crate::handlers::cli::handle_cli_command;
use crate::state::app_state::{ChatState, CliCommandType, ContextMessage, MessageType};

type BoxError = Box<dyn std::error::Error + std::marker::Send + Sync + 'static>;
#[derive(Debug, Serialize, Deserialize)]
pub struct AssistantResponse {
    output: String,
    status: ResponseStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CliResponse {
    output: String,
    status: ResponseStatus,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliCommand {
    command_type: CliCommandType,
    command: String,
}

#[derive(Debug)]
pub enum ChatActionOutcome {
    Continue,
    Stop,
}

pub async fn handle_chat_action(
    typed_msg: ContextMessage,
    chat_state: Arc<ChatState>,
    fe_write_stream: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    cli_write_stream: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
) -> Result<ChatActionOutcome, BoxError> {
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
    if llm_response.status != ResponseStatus::Success {
        // panik
        return Err("err".into());
    }
    // send response to fe stream
    let mut fe_ws = fe_write_stream.lock().await; // Lock the write stream before using it
    fe_ws.send(llm_response.output.into()).await?;
    drop(fe_ws);
    // TODO: send the llm response back to the websocket server's write stream to the FE.

    // update the db by calling dummy_db_function for now; or should the functions the handler hands off to take care of the db updates?
    dummy_db_function();

    // decide if there needs to be any CLI action here
    let command = extract_command_from_frontend_message(typed_msg.content.clone());
    if let Some(command) = command {
        let cli_response = cli_command(command, chat_state.clone()).await;
        match cli_response.status {
            ResponseStatus::Success => {
                let cli_message = ContextMessage {
                    message_type: MessageType::CliOutput,
                    content: cli_response.output,
                    timestamp: Some(chrono::Utc::now()),
                };
                // cli response should automatically be streamed, since the CLI has a websocket connection open with the websocket server.
                let llm_response = openai_message(cli_message, chat_state).await;

                // send response to fe stream
                let mut fe_ws = fe_write_stream.lock().await; // Lock the write stream before using it
                fe_ws.send(llm_response.output.into()).await?;
                drop(fe_ws);
            }
            ResponseStatus::Failure => {
                // do nothing for now
            }
        }
        // TODO: need a way to determine whether a chat outcome should be continue or stop.
        return Ok(ChatActionOutcome::Continue);
    } else {
        // return, the chat action has finished
        // this logic is wrong for now
        return Ok(ChatActionOutcome::Stop);
    }
}

pub async fn openai_message(
    new_message: ContextMessage,
    state: Arc<ChatState>,
) -> AssistantResponse {
    match handle_openai_call(&new_message, &state).await {
        Ok(output) => AssistantResponse {
            output,
            status: ResponseStatus::Success,
        },
        Err(e) => AssistantResponse {
            output: e.to_string(),
            // TODO: make verbose?
            status: ResponseStatus::Failure,
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
            status: ResponseStatus::Success,
        },
        Err(e) => CliResponse {
            output: e.to_string(),
            status: ResponseStatus::Failure,
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
