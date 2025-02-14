use crate::handlers::chat::handle_openai_call_as_mock_user;
use std::collections::HashMap;
use std::fs;
// websocket server entry point
use crate::handlers::handler::{handle_chat_action, openai_message, ChatActionOutcome};
use crate::state::app_state::{
    ChatState, ContextMessage, MessageType, SharedChatState, UserChatPreferences,
};
use actix_web::body::MessageBody;
use futures_util::{SinkExt, StreamExt};
use log::info;
use std::io::Error;
use std::sync::{Arc, Mutex as SyncMutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
use uuid::Uuid;

type ThreadWsMap = Arc<AsyncMutex<HashMap<Uuid, WebSocketPair>>>;
type BoxError = Box<dyn std::error::Error + std::marker::Send + Sync + 'static>;

#[derive(Debug)]
struct WebSocketPair {
    cli_ws: Option<WebSocketStream<TcpStream>>,
    fe_ws: Option<WebSocketStream<TcpStream>>,
}

type ChatActionSender = mpsc::Sender<ChatActionOutcome>;
type ChatActionReceiver = mpsc::Receiver<ChatActionOutcome>;

// websocket server setup
// #[tokio::main] i think this should be removed since tokio main is already in main.rs?
pub async fn start_websocket_server(chat_state: Arc<ChatState>) -> Result<(), Error> {
    let _ = env_logger::try_init();
    let addr = "127.0.0.1:8008".to_string();
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    // we can't guarantee only one thread is only running this server, have to make it multithreaded
    let thread_ws_map: ThreadWsMap = Arc::new(AsyncMutex::new(HashMap::new()));

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
        let mut map = thread_ws_map.lock().await;
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

        if let (Some(fe_ws), Some(cli_ws)) = (entry.fe_ws.take(), entry.cli_ws.take()) {
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

    // Wrap the write streams in Arc<Mutex<>> to allow sharing across async tasks
    let cli_write_stream = Arc::new(AsyncMutex::new(cli_write_stream));
    let fe_write_stream = Arc::new(AsyncMutex::new(fe_write_stream));

    let (auto_run_tx, mut auto_run_rx) = mpsc::channel::<ChatActionOutcome>(1);
    let current_chat_depth: u16 = 0;
    let user_preferences = chat_state.user_preferences.lock().unwrap();
    // let autorun_all = user_preferences.autorun_all;
    // let depth = user_preferences.autorun_all;
    // let autorun_readonly = user_preferences.autorun_readonly;

    let mut autorun_enabled = determine_autorun_status(*user_preferences, current_chat_depth);

    loop {
        tokio::select! {
            Some(fe_msg) = fe_read_stream.next() => {
                if let Ok(msg) = fe_msg {
                    if let Ok(text) = msg.into_text() {
                        current_chat_depth += 1;
                        if let Ok(typed_msg) = serde_json::from_str::<ContextMessage>(&text) {
                            // hand over cli execution to an execution thread
                            println!("FE message type: {:?}", typed_msg.message_type);
                            println!("FE sent: {:?}", typed_msg.content);
                            match typed_msg.message_type {
                                MessageType::UserCancelCmd => {
                                    // TODO: implement kill all children threads of the current thread which are executing chat actions
                                    break;
                                }
                                MessageType::UserPrompt => {
                                    // Handle UserPrompt
                                    autorun_enabled = determine_autorun_status(user_preferences, current_chat_depth);
                                    let chat_state_clone = Arc::clone(&chat_state);
                                    let autorun_tx_clone = auto_run_tx.clone();

                                    tokio::spawn(async move {
                                        let outcome = handle_chat_action(typed_msg, chat_state, fe_write_stream, cli_write_stream).await;
                                        if let Ok(outcome_status) = outcome {
                                            let _ = autorun_tx_clone.send(outcome_status).await;
                                        }
                                    });
                                }
                                MessageType::UserAckCmd => {
                                    // Handle UserAckCmd
                                }
                                _ => {
                                    // Handle other cases
                                    // should throw an err here, since we shouldn't expect to see anything else from the FE
                                }
                            }
                        }
                    }
                }
            },
            // If a message comes from the CLI stream, simply forward it.
            Some(cli_msg) = cli_read_stream.next() => {
                if let Ok(msg) = cli_msg {
                    println!("CLI sent: {:?}", msg);
                    let mut fe_ws = fe_write_stream.lock().await; // Lock the write stream before using it
                    fe_ws.send(msg).await?;
                }
            },

            // this brach receives outcomes from auto-run tasks
            Some(outcome) = auto_run_rx.recv(), if autorun_enabled => {
                match outcome {
                    ChatActionOutcome::Continue => {
                        // TODO: naively mock a user message telling the agent to continue. eventually, this should just be a reasoning step directly
                        // seed the context
                        // seed context window with the mocked user input
                        let next_msg = match get_next_mock_user_message_autorun_mode(&chat_state).await {
                            Ok(completion) => completion,
                            Err(err) => {
                                // quit this
                                err.to_string()
                            }
                        };

                        let typed_msg = ContextMessage {
                            message_type: MessageType::UserPrompt,
                            content: next_msg,
                            timestamp: Some(chrono::Utc::now()),
                        };

                        autorun_enabled = determine_autorun_status(user_preferences, current_chat_depth);
                        let chat_state_clone = Arc::clone(&chat_state);
                        let autorun_tx_clone = auto_run_tx.clone();

                        tokio::spawn(async move {
                            let outcome = handle_chat_action(typed_msg, chat_state, fe_write_stream, cli_write_stream).await;
                            if let Ok(outcome_status) = outcome {
                                let _ = autorun_tx_clone.send(outcome_status).await;
                            }
                        });
                    },
                    ChatActionOutcome::Stop => {
                        // we should hand control back to the frontend, and await further things
                        let mut fe_ws = fe_write_stream.lock().await; // Lock the write stream before using it
                        fe_ws.send(Message::Text("done!".into())).await?;
                    }
                }

            }
        }
    }

    Ok(())
}

// ========================= UTIL FUNCTIONS ==============================
fn determine_autorun_status(
    user_preferences: UserChatPreferences,
    current_chat_depth: u16,
) -> bool {
    let depth = user_preferences.depth;
    let autorun_readonly = user_preferences.autorun_readonly;
    let autorun_all = user_preferences.autorun_all;
    return true;
}

async fn get_next_mock_user_message_autorun_mode(
    state: &ChatState,
) -> Result<String, Box<dyn std::error::Error + 'static>> {
    match handle_openai_call_as_mock_user(state).await {
        Ok(completion) => {
            return Ok(completion);
        }
        Err(err) => {
            eprintln!("Error triggering a mock user message in autorun: {}", err);
            return Err(err.into());
        }
    };
}
