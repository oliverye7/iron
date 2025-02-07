// websocket server entry point
use std::io::Error;
use reqwest::Client;
use futures_util::StreamExt;
use futures_util::SinkExt;
use log::info;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;

// websocket server setup
#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = env_logger::try_init();
    let addr = "127.0.0.1:8008".to_string();
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);
    
    while let Ok((stream, _)) = listener.accept().await {
        let (write, read) = ws_stream.split();
        let mut write = write;
        let mut read = read;
        let uid = read.next();
        if threadmap contains uid {
            // from FE to WS server
            // pass in FE stream and WS stream to controller thread for a connection
            tokio::spawn(accept_connection(stream, threadmap.get(uid))); // only spawn once CLI and FE are up
        } else {
            // TODO: handle error check where user opens invalid FE without CLI first
            threadmap.push(uid, &stream);
        }
    }
    
    Ok(())
}

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

async fn accept_connection(stream: TcpStream) -> Result<(), BoxError> {
    let addr = stream.peer_addr().expect("connected streams should have a peer addr");
    info!("Peer address: {}", addr);
    
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during the websocket handshake");
    info!("New WebSocket connection: {}", addr);
    
    let (write, read) = ws_stream.split();
    let mut write = write;
    let mut read = read;

    while let Some(msg) = read.next().await {
        //spawn a child thread for each chat action
        if msg.type == newUserInput:
            tokio::spawn(newthread) {
        else if type == userAckCmd:
            cli.execute_cached_cmd()
        else if type == userCancelCmd:
            self.kill_all_children()
        else:
            panic
            match msg {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            let modified = format!("{}modified", text);

                            let request_url = "https://jsonplaceholder.typicode.com/posts/1";
                            let client = Client::new();
                            let response = client
                                            .get(request_url)
                                            .send()
                                            .await?;
                            let response_json: serde_json::Value = response.json().await?;
                            let id = &response_json["id"];
                            let title = &response_json["title"];
                            info!("Received dummy text -- id = {} and title = {}", id, title);
                            info!("Received message: {}, sending: {}", text, modified);
                            if let Err(e) = write.send(Message::Text(modified.into())).await {
                                info!("Error sending message: {}", e);
                                break;
                            }
                        },
                        Message::Close(_) => {
                            info!("Client disconnected: {}", addr);
                            execute_cli_command(cmdC);
                            self.kill_all_children();
                            break;
                        },
                        _ => {} // Ignore other message types
                    }
                },
                Err(e) => {
                    info!("Error receiving message from {}: {}", addr, e);
                    break;
                }
            }

        }
    }
    
    info!("Connection closed: {}", addr);
    Ok(())
}