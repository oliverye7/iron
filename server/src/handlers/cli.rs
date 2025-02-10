// cli command handlers
use tokio::process::Command;

use crate::state::app_state::{ChatState, MessageType};

pub async fn handle_cli_command(
    command: String,
    state: &ChatState,
) -> Result<String, Box<dyn std::error::Error + 'static>> {
    // TODO: have this stream instead of running an await on the caller
    state.add_message_to_state(MessageType::CliCommand, command.clone())?;

    let output = Command::new("sh").arg("-c").arg(&command).output().await?;

    let output_str = String::from_utf8(output.stdout)?;

    state.add_message_to_state(MessageType::CliOutput, output_str.clone())?;

    Ok(output_str)
}
