use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
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
    let supported_config = device.default_input_config()?;
    let sample_format = supported_config.sample_format();
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
    let stream = match sample_format {
        cpal::SampleFormat::I16 => {
            build_input_stream::<i16>(&device, &stream_config, Arc::clone(&recognizer), channels)?
        }
        cpal::SampleFormat::U16 => {
            build_input_stream::<u16>(&device, &stream_config, Arc::clone(&recognizer), channels)?
        }
        cpal::SampleFormat::F32 => {
            build_input_stream::<f32>(&device, &stream_config, Arc::clone(&recognizer), channels)?
        }
        _ => return Err("Unsupported sample format".into()),
    };

    stream.play()?;

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    recognizer: Arc<Mutex<Recognizer>>,
    channels: u16,
) -> Result<cpal::Stream, Box<dyn std::error::Error + Send + Sync>>
where
    T: cpal::Sample + cpal::SizedSample,
    i16: cpal::FromSample<T>,
{
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _| {
            let mut samples = Vec::with_capacity(data.len() / channels as usize);
            for frame in data.chunks(channels as usize) {
                let sample = frame
                    .get(0)
                    .copied()
                    .map(i16::from_sample)
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
