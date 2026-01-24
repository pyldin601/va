use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::HostTrait;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Sample, SampleFormat,
};
use tokio::sync::broadcast;
use vosk::{CompleteResult, DecodingState, Model, Recognizer};

pub(crate) fn spawn_shared_recognizer_stream(
    model: Arc<Model>,
    sender: broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::thread::spawn(move || {
        if let Err(err) = run_recognizer_stream(model, sender) {
            eprintln!("audio session error: {err}");
        }
    });
    Ok(())
}

fn run_recognizer_stream(
    model: Arc<Model>,
    sender: broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    let stream = build_input_stream(
        &device,
        &stream_config,
        sample_format,
        recognizer,
        channels,
        sender,
    )?;

    stream.play()?;
    loop {
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn build_input_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: SampleFormat,
    recognizer: Arc<Mutex<Recognizer>>,
    channels: u16,
    sender: broadcast::Sender<String>,
) -> Result<cpal::Stream, Box<dyn std::error::Error + Send + Sync>> {
    match sample_format {
        SampleFormat::I16 => {
            build_input_stream_inner::<i16>(device, config, recognizer, channels, sender)
        }
        SampleFormat::U16 => {
            build_input_stream_inner::<u16>(device, config, recognizer, channels, sender)
        }
        SampleFormat::F32 => {
            build_input_stream_inner::<f32>(device, config, recognizer, channels, sender)
        }
        _ => Err("Unsupported input sample format".into()),
    }
}

fn build_input_stream_inner<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    recognizer: Arc<Mutex<Recognizer>>,
    channels: u16,
    sender: broadcast::Sender<String>,
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
                let sample = frame.get(0).copied().map(i16::from_sample).unwrap_or(0);
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
                            let _ = sender.send(text);
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
    match result {
        CompleteResult::Single(single) => Some(single.text.to_string()),
        CompleteResult::Multiple(multiple) => multiple
            .alternatives
            .first()
            .map(|alternative| alternative.text.to_string()),
    }
}
