use crate::error::Error;
use std::env;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) vosk_model_path: String,
    pub(crate) webhook_url: String,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self, Error> {
        let vosk_model_path =
            env::var("VOSK_MODEL_PATH").map_err(|_| "VOSK_MODEL_PATH is not set")?;

        let webhook_url = env::var("WEBHOOK_URL").map_err(|_| "WEBHOOK_URL is not set")?;

        Ok(Self {
            vosk_model_path,
            webhook_url,
        })
    }
}
