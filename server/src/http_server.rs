// http server entry point
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use std::{os::macos::raw::stat, sync::Mutex};
use std::env;
use std::fs;
use tokio::process::Command;
use serde::Deserialize;
use serde_json::json;
use reqwest::Error;
use reqwest::Client;
use reqwest::header::USER_AGENT;

struct AppState {
    app_name: String,
}

#[derive(Deserialize, Debug)]
struct User {
    login: String,
    id: u32,
}

struct ChatContext {
    context: Mutex<String>, // mutex necessary for sharing across threads
}

#[derive(Deserialize)]
struct ChatInput {
    new_context: String,
}

#[derive(Deserialize)]
struct CliCommand {
    command: String,
}

#[get("/")]
async fn index(data: web::Data<AppState>) -> impl Responder {
    let app_name = &data.app_name;
    format!("Hello {app_name}!")
}

#[post("/openai-message")]
async fn openai_message(
    new_input: web::Json<ChatInput>,
    chat_context: web::Data<ChatContext>
) -> impl Responder {
    // update chat_context to include the new input
    update_context(&new_input.new_context, chat_context);

    match make_openai_call(chat_context).await {
        Ok(response) => {
            update_context(response, chat_context) // function imported from state folder
            return HttpResponse::Ok().body(response);
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[post("/cli-command")]
async fn cli_command(
    cmd: web::Json<CliCommand>,
    prev_context: web::Data<ChatContext>
) -> impl Responder {
    // TODO: add error handling here
    let result = execute_cli_command(&cmd.command);
    update_context(result, prev_context) // imported from state folder

    return HttpResponse::Ok().body(format!("{}", String::from_utf8_lossy(&output.stdout)));
}

// http server setup and routing
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let static_app_state = web::Data::new(AppState {
        app_name: String::from("oliver-bot"),
    });

    let chat_context = web::Data::new(ChatContext {
        context: Mutex::new(String::new())
    });

    HttpServer::new(move || {
        App::new()
            .app_data(static_app_state.clone())
            .app_data(shared_app_state.clone())
            .app_data(chat_context.clone())
            .service(openai_message)
            .service(cli_command)
            .service(index)
    })
    .bind(("127.0.0.1", 6001))?
    .run()
    .await
}
