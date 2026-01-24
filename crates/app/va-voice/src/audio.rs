use std::sync::mpsc::Receiver;
use cpal::traits::HostTrait;

pub(crate) fn create_audio_stream() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device available")?;

    Ok(())
}
