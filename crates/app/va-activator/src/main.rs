mod config;
mod error;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

struct AppState {
    config: Config,
}

#[derive(Deserialize)]
struct WebhookRequest {
    text: String,
}

#[derive(Serialize)]
struct WebhookResponse {
    status: &'static str,
    command: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
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
    info!("va-activator listening on {bind_addr}");

    let app_state = web::Data::new(AppState {
        config,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(health)
            .service(webhook)
    })
    .bind(bind_addr)?
    .run()
    .await
}

#[get("/health")]
async fn health() -> impl Responder {
    web::Json(HealthResponse { status: "ok" })
}

#[post("/webhook")]
async fn webhook(
    state: web::Data<AppState>,
    payload: web::Json<WebhookRequest>,
) -> HttpResponse {
    let text = normalize(&payload.text);
    if text.is_empty() {
        return HttpResponse::Ok().json(WebhookResponse {
            status: "ignored",
            command: None,
        });
    }

    if !starts_with_word(&text, &state.config.activation_word) {
        return HttpResponse::Ok().json(WebhookResponse {
            status: "ignored",
            command: None,
        });
    }

    let command_text = text[state.config.activation_word.len()..].trim();
    if command_text.is_empty() {
        return HttpResponse::Ok().json(WebhookResponse {
            status: "ignored",
            command: None,
        });
    }

    if contains_stop_word(command_text, &state.config.stop_words) {
        info!("stop word detected");
        return HttpResponse::Ok().json(WebhookResponse {
            status: "stopped",
            command: None,
        });
    }

    info!("activation detected");
    HttpResponse::Ok().json(WebhookResponse {
        status: "accepted",
        command: Some(command_text.to_string()),
    })
}

fn normalize(input: &str) -> String {
    input.trim().to_lowercase()
}

fn starts_with_word(text: &str, word: &str) -> bool {
    if text == word {
        return true;
    }
    if !text.starts_with(word) {
        return false;
    }
    text.as_bytes().get(word.len()) == Some(&b' ')
}

fn contains_stop_word(text: &str, stop_words: &HashSet<String>) -> bool {
    text.split_whitespace()
        .any(|token| stop_words.contains(token))
}
