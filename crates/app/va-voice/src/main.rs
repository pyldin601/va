mod audio;
mod config;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::config::get_vosk_model_path;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::Sample;
use vosk::{CompleteResult, DecodingState, Model, Recognizer};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let device = audio::create_device()?;
    let supported_config = device.default_input_config()?;
    let stream_config: cpal::StreamConfig = supported_config.into();

    let sample_rate = stream_config.sample_rate.0 as f32;
    let channels = stream_config.channels;

    let vosk_model_path = get_vosk_model_path()?;
    let vosk_model =
        Model::new(vosk_model_path.to_string_lossy()).ok_or("Failed to load VOSK model")?;
    let vosk_recognizer = Arc::new(Mutex::new(
        Recognizer::new(&vosk_model, sample_rate).ok_or("Failed to create recognizer")?,
    ));

    let stream = device.build_input_stream(
        &stream_config,
        move |data: &[f32], _| {
            let mut samples = Vec::with_capacity(data.len() / channels as usize);

            for frame in data.chunks(channels as usize) {
                let sample = frame.get(0).copied().map(i16::from_sample).unwrap_or(0);
                samples.push(sample);
            }

            let mut recognizer = match vosk_recognizer.lock() {
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

    stream.play()?;

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
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
