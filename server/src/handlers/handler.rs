use std::sync::Arc;

use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;

use crate::handlers::chat::handle_openai_call;
use crate::handlers::cli::handle_cli_command;
use crate::state::app_state::{ChatState, ContextMessage};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct CliCommand {
    command_type: CliCommandType,
    commad: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum CliCommandType {
    ReadOnly,
    WriteExecute,
}

pub async fn handle_chat_action(typed_msg: ContextMessage, chat_state: Arc<ChatState>) -> String {
    "hi".to_string()
}

async fn openai_message(
    new_message: web::Json<ContextMessage>,
    state: web::Data<ChatState>,
) -> impl Responder {
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

#[post("/cli-command")]
async fn cli_command(command: web::Json<String>, state: web::Data<ChatState>) -> impl Responder {
    match handle_cli_command(command.clone(), &state).await {
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
