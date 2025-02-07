use crate::state::{AppState, MessageType};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio_tungstenite::tungstenite::Message as WsMessage;

#[derive(Deserialize)]
pub struct CliInput {
    command: String,
}

#[derive(Serialize)]
pub struct CliResponse {
    output: String,
    status: String,
}

// Shared core logic
pub async fn handle_cli_command(
    command: String,
    state: &AppState,
) -> Result<String, Box<dyn std::error::Error>> {
    // Add command to context
    state.add_message(MessageType::CliCommand, command.clone())?;
    
    // Execute command
    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .await?;
    
    let output_str = String::from_utf8(output.stdout)?;
    
    // Add output to context
    state.add_message(MessageType::CliOutput, output_str.clone())?;
    
    Ok(output_str)
}

// HTTP endpoint wrapper
pub async fn http_cli_handler(
    input: web::Json<CliInput>,
    state: web::Data<AppState>,
) -> HttpResponse {
    match handle_cli_command(input.command.clone(), &state).await {
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

// WebSocket message handler
pub async fn ws_cli_handler(
    command: String,
    state: &AppState,
) -> Result<WsMessage, Box<dyn std::error::Error>> {
    let output = handle_cli_command(command, state).await?;
    Ok(WsMessage::Text(output))
}