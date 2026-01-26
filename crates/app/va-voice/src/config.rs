use crate::error::Error;
use std::env;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) vosk_model_path: String,
    pub(crate) webhook_url: String,
    pub(crate) webhook_queue_size: usize,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self, Error> {
        let vosk_model_path =
            env::var("VOSK_MODEL_PATH").map_err(|_| "VOSK_MODEL_PATH is not set")?;

        let webhook_url = env::var("WEBHOOK_URL").map_err(|_| "WEBHOOK_URL is not set")?;

        let webhook_queue_size = match env::var("WEBHOOK_QUEUE_SIZE") {
            Ok(value) => value
                .parse::<usize>()
                .map_err(|_| "WEBHOOK_QUEUE_SIZE must be a positive integer")?,
            Err(_) => 128,
        };

        Ok(Self {
            vosk_model_path,
            webhook_url,
            webhook_queue_size,
        })
    }
}
