mod config;
mod error;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use crate::config::Config;

struct AppState {
    config: Config,
    client: reqwest::Client,
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
        client: reqwest::Client::new(),
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

    let activation_word = match find_activation_word(&text, &state.config.activation_words) {
        Some(word) => word,
        None => {
            return HttpResponse::Ok().json(WebhookResponse {
                status: "ignored",
                command: None,
            });
        }
    };

    let command_text = text[activation_word.len()..].trim();
    if command_text.is_empty() {
        return HttpResponse::Ok().json(WebhookResponse {
            status: "ignored",
            command: None,
        });
    }

    let command_text = command_text.to_string();
    if contains_stop_word(&command_text, &state.config.stop_words) {
        info!("stop word detected");
        return HttpResponse::Ok().json(WebhookResponse {
            status: "stopped",
            command: None,
        });
    }

    if let Err(err) = forward_command(&state.client, &state.config.next_webhook_url, &command_text).await {
        warn!("webhook forward error: {err:?}");
        return HttpResponse::BadGateway().json(WebhookResponse {
            status: "error",
            command: Some(command_text),
        });
    }

    info!("activation detected");
    HttpResponse::Ok().json(WebhookResponse {
        status: "accepted",
        command: Some(command_text),
    })
}

fn normalize(input: &str) -> String {
    input.trim().to_lowercase()
}

fn find_activation_word<'a>(text: &str, words: &'a [String]) -> Option<&'a str> {
    for word in words {
        if text == word {
            return Some(word.as_str());
        }
        if text.starts_with(word) && text.as_bytes().get(word.len()) == Some(&b' ') {
            return Some(word.as_str());
        }
    }
    None
}

fn contains_stop_word(text: &str, stop_words: &HashSet<String>) -> bool {
    text.split_whitespace()
        .any(|token| stop_words.contains(token))
}

async fn forward_command(
    client: &reqwest::Client,
    webhook_url: &str,
    command: &str,
) -> Result<(), reqwest::Error> {
    client
        .post(webhook_url)
        .json(&serde_json::json!({ "text": command }))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
