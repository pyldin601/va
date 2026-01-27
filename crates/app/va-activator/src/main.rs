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

    if let Err(err) = forward_command(&state.client, &state.config.webhook_url, &command_text).await {
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

fn find_activation_word<'a>(text: &str, words: &'a HashSet<String>) -> Option<&'a str> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse, HttpServer};
    use std::sync::{Arc, Mutex};
    use std::net::TcpListener;

    #[derive(Deserialize)]
    struct DownstreamRequest {
        text: String,
    }

    fn test_config(webhook_url: String) -> Config {
        Config {
            activation_words: ["va".to_string(), "assistant".to_string()]
                .into_iter()
                .collect(),
            stop_words: ["stop".to_string(), "cancel".to_string()]
                .into_iter()
                .collect(),
            bind_addr: "127.0.0.1:0".to_string(),
            webhook_url,
        }
    }

    async fn start_downstream() -> (String, Arc<Mutex<Vec<String>>>, actix_web::dev::ServerHandle) {
        let received = Arc::new(Mutex::new(Vec::<String>::new()));
        let received_clone = received.clone();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let server = HttpServer::new(move || {
            let received = received_clone.clone();
            App::new().route(
                "/webhook",
                web::post().to(move |payload: web::Json<DownstreamRequest>| {
                    let received = received.clone();
                    async move {
                        received.lock().unwrap().push(payload.text.clone());
                        HttpResponse::Ok().finish()
                    }
                }),
            )
        })
        .listen(listener)
        .unwrap()
        .run();

        let handle = server.handle();
        actix_web::rt::spawn(server);

        (format!("http://{addr}/webhook"), received, handle)
    }

    #[actix_web::test]
    async fn forwards_command_for_any_activation_word() {
        let (downstream_url, received, handle) = start_downstream().await;

        let config = test_config(downstream_url);
        let app_state = web::Data::new(AppState {
            config,
            client: reqwest::Client::new(),
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .service(webhook),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/webhook")
            .set_json(&serde_json::json!({ "text": "assistant play music" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let forwarded = received.lock().unwrap();
        assert_eq!(forwarded.as_slice(), &["play music"]);

        handle.stop(true).await;
    }

    #[actix_web::test]
    async fn ignores_empty_command_after_activation_word() {
        let (downstream_url, _received, handle) = start_downstream().await;
        let config = test_config(downstream_url);
        let app_state = web::Data::new(AppState {
            config,
            client: reqwest::Client::new(),
        });
        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .service(webhook),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/webhook")
            .set_json(&serde_json::json!({ "text": "va" }))
            .to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp["status"], "ignored");
        assert!(resp["command"].is_null());

        handle.stop(true).await;
    }

    #[actix_web::test]
    async fn stops_on_stop_word_and_does_not_forward() {
        let (downstream_url, received, handle) = start_downstream().await;
        let config = test_config(downstream_url);
        let app_state = web::Data::new(AppState {
            config,
            client: reqwest::Client::new(),
        });
        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .service(webhook),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/webhook")
            .set_json(&serde_json::json!({ "text": "va cancel the alarm" }))
            .to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp["status"], "stopped");
        assert!(resp["command"].is_null());

        let forwarded = received.lock().unwrap();
        assert!(forwarded.is_empty());

        handle.stop(true).await;
    }
}
