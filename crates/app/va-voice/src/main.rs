mod audio;

use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use vosk::{CompleteResult, DecodingState, Model, Recognizer};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let model_path = model_path()?;
    run_vosk(model_path)
}

fn model_path() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let path = env::var("VOSK_MODEL_PATH")
        .map(PathBuf::from)
        .map_err(|_| "VOSK_MODEL_PATH is not set")?;
    Ok(path)
}

fn run_vosk(model_path: PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let model_path = model_path.to_str().ok_or("Invalid VOSK model path")?;
    let model = Model::new(model_path).ok_or("Failed to load VOSK model")?;

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device available")?;
    let supported_config = select_i16_mono_48k_config(&device)?;
    let stream_config: cpal::StreamConfig = supported_config.into();
    let sample_rate = stream_config.sample_rate.0 as f32;
    let channels = stream_config.channels;
    let recognizer = Arc::new(Mutex::new(
        Recognizer::new(&model, sample_rate).ok_or("Failed to create recognizer")?,
    ));

    println!(
        "listening on {} ({} Hz, {}ch)",
        device
            .name()
            .unwrap_or_else(|_| "default input".to_string()),
        sample_rate as u32,
        channels
    );
    let stream =
        build_input_stream(&device, &stream_config, Arc::clone(&recognizer), channels)?;

    stream.play()?;

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn select_i16_mono_48k_config(
    device: &cpal::Device,
) -> Result<cpal::SupportedStreamConfig, Box<dyn std::error::Error + Send + Sync>> {
    let mut supported_configs = device.supported_input_configs()?;
    let target_rate = cpal::SampleRate(48_000);
    let config = supported_configs.find(|config| {
        config.sample_format() == cpal::SampleFormat::I16
            && config.channels() == 1
            && config.min_sample_rate() <= target_rate
            && config.max_sample_rate() >= target_rate
    });
    match config {
        Some(config) => Ok(config.with_sample_rate(target_rate)),
        None => Err("Device does not support i16 mono 48000 Hz input".into()),
    }
}

fn build_input_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    recognizer: Arc<Mutex<Recognizer>>,
    channels: u16,
) -> Result<cpal::Stream, Box<dyn std::error::Error + Send + Sync>>
{
    let stream = device.build_input_stream(
        config,
        move |data: &[i16], _| {
            let mut samples = Vec::with_capacity(data.len() / channels as usize);
            for frame in data.chunks(channels as usize) {
                let sample = frame
                    .get(0)
                    .copied()
                    .unwrap_or(0);
                samples.push(sample);
            }
            let mut recognizer = match recognizer.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };
            match recognizer.accept_waveform(&samples) {
                Ok(DecodingState::Finalized) => {
                    if let Some(text) = complete_text(recognizer.result()) {
                        if !text.is_empty() {
                            println!("{text}");
                        }
                    }
                }
                Ok(DecodingState::Failed) => {
                    eprintln!("decoding failed");
                }
                Ok(DecodingState::Running) => {}
                Err(err) => eprintln!("decode error: {err}"),
            }
        },
        move |err| eprintln!("audio error: {err}"),
        None,
    )?;

    Ok(stream)
}

fn complete_text(result: CompleteResult<'_>) -> Option<String> {
    eprintln!("RES: {result:?}");
    match result {
        CompleteResult::Single(single) => Some(single.text.to_string()),
        CompleteResult::Multiple(multiple) => multiple
            .alternatives
            .first()
            .map(|alternative| alternative.text.to_string()),
    }
}
