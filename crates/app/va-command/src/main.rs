mod config;
mod error;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use crate::config::Config;

const MAX_INPUT_LENGTH: usize = 20000;

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
    message: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
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
    info!("va-command listening on {bind_addr}");

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
    let command = payload.text.trim();
    if command.is_empty() {
        return HttpResponse::BadRequest().json(WebhookResponse {
            status: "error",
            message: "Missing command text".to_string(),
        });
    }

    if command.len() > MAX_INPUT_LENGTH {
        return HttpResponse::BadRequest().json(WebhookResponse {
            status: "error",
            message: format!("Command exceeds {MAX_INPUT_LENGTH} characters"),
        });
    }

    let prompt = build_prompt(command);
    if let Err(err) = send_to_ollama(&state, &prompt).await {
        warn!("ollama error: {err:?}");
    }

    HttpResponse::Ok().json(WebhookResponse {
        status: "ok",
        message: format!("User asked about {command}"),
    })
}

fn build_prompt(command: &str) -> String {
    format!(
        "You are a command handler for a voice assistant. \
Return a short single sentence describing what the user asked about.\n\n\
Example:\n\
User: \"turn on the living room lights\"\n\
Assistant: \"User asked about turning on the living room lights.\"\n\n\
User: \"{command}\"\n\
Assistant:",
    )
}

async fn send_to_ollama(state: &AppState, prompt: &str) -> Result<(), reqwest::Error> {
    let base_url = state.config.ollama_base_url.trim_end_matches('/');
    let url = format!("{base_url}/api/generate");
    let payload = OllamaRequest {
        model: &state.config.ollama_model,
        prompt,
        stream: false,
    };

    let response = state
        .client
        .post(url)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    let body: OllamaResponse = response.json().await?;
    eprintln!("ollama response: {}", body.response);
    Ok(())
}
