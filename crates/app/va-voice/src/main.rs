use std::time::{Duration, SystemTime, UNIX_EPOCH};

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use bytes::Bytes;
use futures_util::stream::{self, StreamExt};
use serde::Serialize;

#[derive(Clone, Copy)]
struct TextEvent<'a> {
    ts_ms: u64,
    text: &'a str,
    kind: &'a str,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    started_at_ms: u64,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080";
    println!("va-voice listening on {addr}");

    HttpServer::new(|| App::new().service(health).service(text))
        .bind(addr)?
        .run()
        .await
}

#[get("/health")]
async fn health() -> impl Responder {
    web::Json(HealthResponse {
        status: "ok",
        started_at_ms: now_ms(),
    })
}

#[get("/text")]
async fn text() -> HttpResponse {
    let started_at = now_ms();
    let events = [
        TextEvent {
            ts_ms: started_at,
            text: "listening",
            kind: "partial",
        },
        TextEvent {
            ts_ms: started_at + 600,
            text: "listening to microphone",
            kind: "final",
        },
        TextEvent {
            ts_ms: started_at + 1200,
            text: "vosk engine not wired yet",
            kind: "partial",
        },
        TextEvent {
            ts_ms: started_at + 1800,
            text: "ready for speech input",
            kind: "final",
        },
    ];

    let stream = stream::iter(events).then(|event| async move {
        tokio::time::sleep(Duration::from_millis(750)).await;
        let payload = format!(
            r#"{{"ts_ms":{},"text":"{}","kind":"{}"}}"#,
            event.ts_ms, event.text, event.kind
        );
        let message = format!("event: {}\ndata: {}\n\n", event.kind, payload);
        Ok::<Bytes, actix_web::Error>(Bytes::from(message))
    });

    HttpResponse::Ok()
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .content_type("text/event-stream")
        .streaming(stream)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
