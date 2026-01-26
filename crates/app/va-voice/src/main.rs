use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use bytes::Bytes;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use futures_util::stream::StreamExt;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;
use tracing_subscriber::EnvFilter;
use vosk::{Model, Recognizer};

#[derive(Clone)]
struct TextEvent {
    ts_ms: u64,
    text: String,
    kind: EventKind,
}

#[derive(Clone, Copy)]
enum EventKind {
    Partial,
    Final,
    Error,
}

impl EventKind {
    fn as_str(self) -> &'static str {
        match self {
            EventKind::Partial => "partial",
            EventKind::Final => "final",
            EventKind::Error => "error",
        }
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    started_at_ms: u64,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080";
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
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
    let (sender, receiver) = mpsc::channel::<TextEvent>(64);
    let model_path = model_path();
    std::thread::spawn(move || {
        if let Err(error) = run_vosk(sender.clone(), model_path) {
            let _ = sender.blocking_send(TextEvent {
                ts_ms: now_ms(),
                text: error.to_string(),
                kind: EventKind::Error,
            });
        }
    });

    let stream = ReceiverStream::new(receiver).map(|event| {
        let payload = format!(
            r#"{{"ts_ms":{},"text":"{}","kind":"{}"}}"#,
            event.ts_ms,
            event.text,
            event.kind.as_str()
        );
        let message = format!("event: {}\ndata: {}\n\n", event.kind.as_str(), payload);
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

fn model_path() -> PathBuf {
    env::var("VOSK_MODEL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/opt/vosk-model"))
}

fn run_vosk(
    sender: mpsc::Sender<TextEvent>,
    model_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let model_path = model_path.to_str().ok_or("Invalid VOSK model path")?;
    let model = Model::new(model_path)?;

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device available")?;
    let supported_config = device.default_input_config()?;
    let sample_format = supported_config.sample_format();
    let stream_config: cpal::StreamConfig = supported_config.into();
    let sample_rate = stream_config.sample_rate.0 as f32;
    let channels = stream_config.channels;
    let recognizer = Arc::new(Mutex::new(Recognizer::new(&model, sample_rate)?));

    let ready = TextEvent {
        ts_ms: now_ms(),
        text: format!(
            "listening on {} ({} Hz, {}ch)",
            device
                .name()
                .unwrap_or_else(|_| "default input".to_string()),
            sample_rate as u32,
            channels
        ),
        kind: EventKind::Final,
    };
    let _ = sender.blocking_send(ready);

    let error_sender = sender.clone();
    let stream = match sample_format {
        cpal::SampleFormat::I16 => build_input_stream::<i16>(
            &device,
            &stream_config,
            Arc::clone(&recognizer),
            sender,
            channels,
        )?,
        cpal::SampleFormat::U16 => build_input_stream::<u16>(
            &device,
            &stream_config,
            Arc::clone(&recognizer),
            sender,
            channels,
        )?,
        cpal::SampleFormat::F32 => build_input_stream::<f32>(
            &device,
            &stream_config,
            Arc::clone(&recognizer),
            sender,
            channels,
        )?,
        _ => return Err("Unsupported sample format".into()),
    };

    stream.play()?;

    loop {
        std::thread::sleep(Duration::from_secs(1));
        if error_sender.is_closed() {
            break;
        }
    }

    Ok(())
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    recognizer: Arc<Mutex<Recognizer>>,
    sender: mpsc::Sender<TextEvent>,
    channels: u16,
) -> Result<cpal::Stream, Box<dyn std::error::Error + Send + Sync>>
where
    T: cpal::Sample,
{
    let err_sender = sender.clone();
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _| {
            let mut samples = Vec::with_capacity(data.len() / channels as usize);
            for frame in data.chunks(channels as usize) {
                let sample = frame
                    .get(0)
                    .copied()
                    .map(cpal::Sample::to_f32)
                    .unwrap_or(0.0);
                samples.push((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
            }
            let mut recognizer = match recognizer.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };
            if recognizer.accept_waveform(&samples) {
                if let Some(text) = extract_text(&recognizer.result(), "text") {
                    if !text.is_empty() {
                        info!("recognized: {text}");
                    }
                    send_event(&sender, EventKind::Final, text);
                }
            } else if let Some(text) = extract_text(&recognizer.partial_result(), "partial") {
                if !text.is_empty() {
                    send_event(&sender, EventKind::Partial, text);
                }
            }
        },
        move |err| {
            let _ = err_sender.blocking_send(TextEvent {
                ts_ms: now_ms(),
                text: format!("audio error: {err}"),
                kind: EventKind::Error,
            });
        },
        None,
    )?;

    Ok(stream)
}

fn extract_text(payload: &str, key: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(payload).ok()?;
    value.get(key)?.as_str().map(str::to_string)
}

fn send_event(sender: &mpsc::Sender<TextEvent>, kind: EventKind, text: String) {
    let _ = sender.blocking_send(TextEvent {
        ts_ms: now_ms(),
        text,
        kind,
    });
}
