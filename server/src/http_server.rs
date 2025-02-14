// http server entry point
use actix_web::{get, web, App, HttpServer, Responder};
use std::sync::Mutex;
use uuid::Uuid;

use crate::state::app_state::{ChatState, SharedChatState};

#[get("/")]
async fn index() -> impl Responder {
    format!("Hello world from iron!")
}

// probably need some other functions here for stripe plans ad stuff, but the core logic shouldnt be on this server

// http server setup and routing
// #[actix_web::main]
pub async fn start_http_server() -> std::io::Result<()> {
    HttpServer::new(move || App::new().service(index))
        .bind(("127.0.0.1", 6001))?
        .run()
        .await
}
