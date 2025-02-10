use std::sync::Arc;

use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;

use crate::handlers::chat::handle_openai_call;
use crate::handlers::cli::handle_cli_command;
use crate::state::app_state::{ChatState, CliCommandType, ContextMessage, MessageType};

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

pub async fn handle_chat_action(typed_msg: ContextMessage, chat_state: Arc<ChatState>) -> String {
    let llm_response = openai_message(typed_msg, state);
    return "Ok".to_string();
}

async fn openai_message(new_message: ContextMessage, state: Arc<ChatState>) -> impl Responder {
    match handle_openai_call(&new_message, &state).await {
        Ok(output) => HttpResponse::Ok().json(AssistantResponse {
            output,
            status: "success".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(AssistantResponse {
            output: e.to_string(),
            status: "error".to_string(),
        }),
    }
}

async fn cli_command(command: web::Json<CliCommand>, state: Arc<ChatState>) -> impl Responder {
    match handle_cli_command(
        command.command.clone(),
        command.command_type.clone(),
        &state,
    )
    .await
    {
        Ok(output) => HttpResponse::Ok().json(CliResponse {
            output,
            status: "success".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(CliResponse {
            output: e.to_string(),
            status: "error".to_string(),
        }),
    }
}
