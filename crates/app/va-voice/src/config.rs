use crate::error::Error;
use std::env;

const ENV_VOSK_MODEL_PATH: &str = "VOSK_MODEL_PATH";
const ENV_WEBHOOK_URL: &str = "WEBHOOK_URL";
const ENV_WEBHOOK_QUEUE_SIZE: &str = "WEBHOOK_QUEUE_SIZE";

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) vosk_model_path: String,
    pub(crate) webhook_url: String,
    pub(crate) webhook_queue_size: usize,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self, Error> {
        let vosk_model_path = env::var(ENV_VOSK_MODEL_PATH)
            .map_err(|_| format!("{ENV_VOSK_MODEL_PATH} is not set"))?;

        let webhook_url =
            env::var(ENV_WEBHOOK_URL).map_err(|_| format!("{ENV_WEBHOOK_URL} is not set"))?;

        let webhook_queue_size = match env::var(ENV_WEBHOOK_QUEUE_SIZE) {
            Ok(value) => value
                .parse::<usize>()
                .map_err(|_| format!("{ENV_WEBHOOK_QUEUE_SIZE} must be a positive integer"))?,
            Err(_) => 128,
        };

        Ok(Self {
            vosk_model_path,
            webhook_url,
            webhook_queue_size,
        })
    }
}
