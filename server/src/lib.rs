// shared library code -- not sure if this needs to contain anything yet?
pub mod handlers;
pub mod http_server;
pub mod main;
pub mod state;
pub mod websocket_server;

pub fn extract_command_from_frontend_message(command: String) -> String {
    return command;
}
