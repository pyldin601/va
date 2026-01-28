mod config;
mod error;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use chrono::Local;
use serde::Serialize;
use serde_json::json;
use tracing::info;
use tracing_subscriber::EnvFilter;
use va_skills::{
    CommandRequest, CommandResponse, DateNowResult, ExecuteRequest, TimeNowResult,
    COMMAND_DATE_NOW, COMMAND_TIME_NOW,
};

use crate::config::Config;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct CommandDescriptor {
    command: &'static str,
    description: &'static str,
    example_request: serde_json::Value,
    example_response: serde_json::Value,
}

#[derive(Serialize)]
struct CommandsResponse {
    instructions: &'static str,
    commands: Vec<CommandDescriptor>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("config error: {err}");
            std::process::exit(1);
        }
    };

    let bind_addr = config.bind_addr.clone();
    info!("va-actions listening on {bind_addr}");

    HttpServer::new(move || App::new().service(health).service(commands).service(execute))
        .bind(bind_addr)?
        .run()
        .await
}

#[get("/health")]
async fn health() -> impl Responder {
    web::Json(HealthResponse { status: "ok" })
}

#[get("/commands")]
async fn commands() -> impl Responder {
    let now = Local::now();
    let time_now = CommandDescriptor {
        command: COMMAND_TIME_NOW,
        description: "Return the server's current local time.",
        example_request: json!({ "command": COMMAND_TIME_NOW }),
        example_response: json!({
            "command": COMMAND_TIME_NOW,
            "result": {
                "time": now.format("%H:%M:%S").to_string(),
                "rfc3339": now.to_rfc3339()
            }
        }),
    };

    let date_now = CommandDescriptor {
        command: COMMAND_DATE_NOW,
        description: "Return the server's current local date.",
        example_request: json!({ "command": COMMAND_DATE_NOW }),
        example_response: json!({
            "command": COMMAND_DATE_NOW,
            "result": {
                "date": now.format("%Y-%m-%d").to_string(),
                "rfc3339": now.to_rfc3339()
            }
        }),
    };

    let response = CommandsResponse {
        instructions: "POST /execute with a JSON body containing {\"command\": \"...\"}. Each command returns a result object shaped for that command.",
        commands: vec![time_now, date_now],
    };

    web::Json(response)
}

#[post("/execute")]
async fn execute(payload: web::Json<ExecuteRequest>) -> HttpResponse {
    let now = Local::now();
    let response = match payload.command {
        CommandRequest::TimeNow => CommandResponse::TimeNow(TimeNowResult {
            time: now.format("%H:%M:%S").to_string(),
            rfc3339: now.to_rfc3339(),
        }),
        CommandRequest::DateNow => CommandResponse::DateNow(DateNowResult {
            date: now.format("%Y-%m-%d").to_string(),
            rfc3339: now.to_rfc3339(),
        }),
    };

    HttpResponse::Ok().json(response)
}
