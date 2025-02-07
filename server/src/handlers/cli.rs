// cli command handlers
use tokio::process::Command;

pub async handle_cli_command(
    command: String,
    state: &AppState,
) -> Result<String, Box<dyn std::error::Error>> {
    state.add_message_to_state(MessageType::CliCommand, command.clone())?;

    let output = Command::new("sh")
                    .arg("-c")
                    .arg(&cmd.command)
                    .output()
                    .await?;
    
    
    
}
async fn execute_cli_command(command: String) -> String {
    // TODO: handle errors
    // TODO: prevent malicous commands somehow
    let output = Command::new("sh")
                    .arg("-c")
                    .arg(&cmd.command)
                    .output()
                    .await
                    .expect("failed to execute process");

    return &output.stdout;
}