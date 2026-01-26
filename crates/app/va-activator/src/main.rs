use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::sync::Mutex;
use tracing::info;
use tracing_subscriber::EnvFilter;

const ENV_ACTIVATION_WORD: &str = "ACTIVATION_WORD";
const ENV_STOP_WORDS: &str = "STOP_WORDS";
const ENV_BIND_ADDR: &str = "BIND_ADDR";

#[derive(Clone)]
struct Config {
    activation_word: String,
    stop_words: HashSet<String>,
    bind_addr: String,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let activation_word = env::var(ENV_ACTIVATION_WORD)
            .map_err(|_| format!("{ENV_ACTIVATION_WORD} is not set"))?
            .trim()
            .to_lowercase();
        if activation_word.is_empty() {
            return Err(format!("{ENV_ACTIVATION_WORD} must not be empty"));
        }

        let stop_words_raw = env::var(ENV_STOP_WORDS)
            .map_err(|_| format!("{ENV_STOP_WORDS} is not set"))?;
        let stop_words = stop_words_raw
            .split(',')
            .map(|word| word.trim().to_lowercase())
            .filter(|word| !word.is_empty())
            .collect::<HashSet<_>>();
        if stop_words.is_empty() {
            return Err(format!("{ENV_STOP_WORDS} must contain at least one word"));
        }

        let bind_addr = env::var(ENV_BIND_ADDR).unwrap_or_else(|_| "127.0.0.1:8090".to_string());

        Ok(Self {
            activation_word,
            stop_words,
            bind_addr,
        })
    }
}

#[derive(Default)]
struct ListenerState {
    listening: bool,
    buffer: Vec<String>,
}

struct AppState {
    config: Config,
    listener: Mutex<ListenerState>,
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
        listener: Mutex::new(ListenerState::default()),
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

    let mut listener = match state.listener.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return HttpResponse::InternalServerError().json(WebhookResponse {
                status: "error",
                command: None,
            });
        }
    };

    if !listener.listening {
        if starts_with_word(&text, &state.config.activation_word) {
            listener.listening = true;
            let remainder = text[state.config.activation_word.len()..].trim();
            if !remainder.is_empty() {
                if state.config.stop_words.contains(remainder) {
                    listener.listening = false;
                    return HttpResponse::Ok().json(WebhookResponse {
                        status: "stopped",
                        command: None,
                    });
                }
                listener.buffer.push(remainder.to_string());
            }
            info!("activation detected");
            return HttpResponse::Ok().json(WebhookResponse {
                status: "listening",
                command: None,
            });
        }

        return HttpResponse::Ok().json(WebhookResponse {
            status: "ignored",
            command: None,
        });
    }

    if state.config.stop_words.contains(&text) {
        let command = if listener.buffer.is_empty() {
            None
        } else {
            Some(listener.buffer.join(" "))
        };
        listener.listening = false;
        listener.buffer.clear();
        info!("stop word detected");
        return HttpResponse::Ok().json(WebhookResponse {
            status: "stopped",
            command,
        });
    }

    listener.buffer.push(text);
    HttpResponse::Ok().json(WebhookResponse {
        status: "capturing",
        command: None,
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
