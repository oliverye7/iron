use std::sync::{Arc, Mutex};

use uuid::Uuid;

use server::http_server::start_http_server;
use server::state::app_state::{ChatState, SharedChatState};
use server::websocket_server::start_websocket_server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let chat_state: SharedChatState = Arc::new(ChatState {
        chat_id: Uuid::new_v4(),
        chat_context: Mutex::new(Vec::new()),
    });

    let chat_state_clone = Arc::clone(&chat_state);

    tokio::spawn(async move {
        if let Err(e) = start_websocket_server(chat_state_clone).await {
            eprintln!("WebSocket server error: {}", e);
        }
    });

    start_http_server().await?;
    Ok(())
}
