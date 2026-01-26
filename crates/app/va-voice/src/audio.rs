use std::sync::{mpsc, Arc, Mutex};

use crate::error::Error;
use cpal::{
    traits::DeviceTrait, Device, FromSample, Sample, SampleFormat, SizedSample, Stream,
    StreamConfig,
};
use tracing::{error, warn};
use vosk::{CompleteResult, DecodingState, Recognizer};

pub(crate) fn build_input_stream(
    device: &Device,
    config: &StreamConfig,
    sample_format: SampleFormat,
    recognizer: Arc<Mutex<Recognizer>>,
    channels: u16,
    sender: mpsc::Sender<String>,
) -> Result<Stream, Error> {
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
    device: &Device,
    config: &StreamConfig,
    recognizer: Arc<Mutex<Recognizer>>,
    channels: u16,
    sender: mpsc::Sender<String>,
) -> Result<Stream, Error>
where
    T: Sample + SizedSample,
    i16: FromSample<T>,
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
                    warn!("decoding failed");
                }
                Ok(DecodingState::Running) => {}
                Err(err) => error!("decode error: {err}"),
            }
        },
        move |err| error!("audio error: {err}"),
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
