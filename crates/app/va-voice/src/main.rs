mod audio;
mod config;
mod error;
mod setup;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync;
use tracing::warn;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    let config = config::Config::from_env()?;

    let (sender, receiver) = sync::mpsc::sync_channel::<String>(config.webhook_queue_size);

    let t = std::thread::spawn({
        let client = reqwest::blocking::Client::new();
        let webhook_url = config.webhook_url.clone();

        move || {
            while let Ok(text) = receiver.recv() {
                let result = client
                    .post(&webhook_url)
                    .json(&serde_json::json!({ "text": text }))
                    .send();

                if let Err(err) = result {
                    warn!("webhook error: {err:?}")
                }
            }
        }
    });

    let model = setup::setup_vosk_model(&config)?;
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device available")?;
    let supported_config = device.default_input_config()?;
    let sample_format = supported_config.sample_format();
    let stream_config: cpal::StreamConfig = supported_config.into();
    let sample_rate = stream_config.sample_rate as f32;
    let channels = stream_config.channels;

    let recognizer = setup::setup_recognizer(&model, sample_rate)?;

    let stream = audio::build_input_stream(
        &device,
        &stream_config,
        sample_format,
        recognizer,
        channels,
        sender,
    )?;

    stream.play()?;

    t.join().expect("thread panicked");

    Ok(())
}
