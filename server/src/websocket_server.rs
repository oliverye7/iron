use std::collections::HashMap;
// websocket server entry point
use crate::extract_command_from_frontend_message;
use crate::handlers::handler::handle_chat_action;
use crate::state::app_state::{ChatState, ContextMessage, MessageType, SharedChatState};
use futures_util::{SinkExt, StreamExt};
use log::info;
use std::io::Error;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
use uuid::Uuid;

type ThreadWsMap = Arc<Mutex<HashMap<Uuid, WebSocketPair>>>;
type BoxError = Box<dyn std::error::Error + std::marker::Send + Sync + 'static>;

#[derive(Debug)]
struct WebSocketPair {
    cli_ws: Option<WebSocketStream<TcpStream>>,
    fe_ws: Option<WebSocketStream<TcpStream>>,
}

// websocket server setup
// #[tokio::main] i think this should be removed since tokio main is already in main.rs?
pub async fn start_websocket_server(chat_state: Arc<ChatState>) -> Result<(), Error> {
    let _ = env_logger::try_init();
    let addr = "127.0.0.1:8008".to_string();
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    // we can't guarantee only one thread is only running this server, have to make it multithreaded
    let thread_ws_map: ThreadWsMap = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, _)) = listener.accept().await {
        // stream ==> websocket
        // have thread manage two streams (ws) for cli tool and frontend pubsub
        // TODO: is this really janky and memory inefficient?
        // if i don't clone then rust compalins about borrowed memory but this seems
        // expensive if there are ~1k+ connections or smt
        let chat_state = Arc::clone(&chat_state); // Clone for each new connection

        // or do i have to use Arc::clone(&thread_ws_map)?
        let thread_ws_map = thread_ws_map.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, thread_ws_map, chat_state).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    stream: TcpStream,
    thread_ws_map: ThreadWsMap,
    chat_state: SharedChatState,
) -> Result<(), BoxError> {
    let addr = stream.peer_addr()?;
    info!("New websocket peer address from: {}", addr);

    let ws_stream = accept_async(stream).await?;

    let (write, mut read) = ws_stream.split();

    // we can assume that the FE and CLI all have the same UUID
    // - either the websocket server can create the UUID, or FE can create it
    // - in this case, assume CLI (which starts up first), will send the UUID as its first message
    // - this therefore removes the need to use the tid to manage mappings :)
    let id = match read.next().await {
        Some(Ok(Message::Text(text))) => Uuid::parse_str(&text).unwrap_or(Uuid::new_v4()),
        _ => Uuid::new_v4(),
    };
    // TODO: good to have in the above a second message so we can confidently identify if the id is coming from
    // the CLI tool or the FE, in the off chance that there is unintended behavior of the FE opening a connection first

    // we can also assume that for any pair of websockets we want a thread to manage, the cli websocket is *always*
    // connected first (since the user initiates via cli)
    {
        // create a new scope such that the lock on thread_ws_map is automatically released when the scope is exited
        let mut map = thread_ws_map.lock().unwrap();
        let entry = map.entry(id).or_insert(WebSocketPair {
            cli_ws: None,
            fe_ws: None,
        });

        if entry.cli_ws.is_none() {
            println!("CLI connected for session: {}", id);
            entry.cli_ws = Some(write.reunite(read).unwrap());
        } else if entry.fe_ws.is_none() {
            println!("Frontend connected for session: {}", id);
            entry.fe_ws = Some(write.reunite(read).unwrap());
        }

        if entry.cli_ws.is_some() && entry.fe_ws.is_some() {
            // we're in business, spawn a new child thread to handle the pair of websockets
            println!("Paired CLI and FE, starting handler...");
            let pair = map.remove(&id).unwrap();
            tokio::spawn(handle_cli_fe_pair(pair, chat_state));
        }
    }

    info!("Connection closed: {}", addr);
    Ok(())
}

async fn handle_cli_fe_pair(
    cli_fe_pair: WebSocketPair,
    chat_state: Arc<ChatState>,
) -> Result<(), BoxError> {
    // Your websocket handling logic here
    let (mut cli_write_stream, mut cli_read_stream) = cli_fe_pair.cli_ws.unwrap().split();
    let (mut fe_write_stream, mut fe_read_stream) = cli_fe_pair.fe_ws.unwrap().split();

    loop {
        tokio::select! {
            Some(fe_msg) = fe_read_stream.next() => {
                if let Ok(msg) = fe_msg {
                    if let Ok(text) = msg.into_text() {
                        if let Ok(typed_msg) = serde_json::from_str::<ContextMessage>(&text) {
                            // hand over cli execution to an execution thread
                            println!("FE message type: {:?}", typed_msg.message_type);
                            println!("FE sent: {:?}", typed_msg.content);
                            if typed_msg.message_type == MessageType::UserCancelCmd {
                                // TODO: implement kill all children threads of the current thread which are executing chat actions
                                break;
                            }

                            let chat_state = Arc::clone(&chat_state);
                            // we should call the cli command handler here...
                            tokio::spawn(handle_chat_action(&typed_msg, chat_state));

                        }
                    }
                }
            },
            Some(cli_msg) = cli_read_stream.next() => {
                if let Ok(msg) = cli_msg {
                    println!("CLI sent: {:?}", msg);
                    fe_write_stream.send(msg).await?;
                }
            },
        }
    }

    Ok(())
}
