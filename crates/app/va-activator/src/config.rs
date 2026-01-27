use std::collections::HashSet;
use std::env;

use crate::error::Error;

const ENV_ACTIVATION_WORDS: &str = "ACTIVATION_WORDS";
const ENV_STOP_WORDS: &str = "STOP_WORDS";
const ENV_BIND_ADDR: &str = "BIND_ADDR";
const ENV_WEBHOOK_URL: &str = "WEBHOOK_URL";

#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) activation_words: Vec<String>,
    pub(crate) stop_words: HashSet<String>,
    pub(crate) bind_addr: String,
    pub(crate) next_webhook_url: String,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self, Error> {
        let activation_words_raw = env::var(ENV_ACTIVATION_WORDS)
            .map_err(|_| format!("{ENV_ACTIVATION_WORDS} is not set"))?;
        let activation_words = activation_words_raw
            .split(',')
            .map(|word| word.trim().to_lowercase())
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();
        if activation_words.is_empty() {
            return Err(format!("{ENV_ACTIVATION_WORDS} must contain at least one word").into());
        }

        let stop_words_raw =
            env::var(ENV_STOP_WORDS).map_err(|_| format!("{ENV_STOP_WORDS} is not set"))?;
        let stop_words = stop_words_raw
            .split(',')
            .map(|word| word.trim().to_lowercase())
            .filter(|word| !word.is_empty())
            .collect::<HashSet<_>>();
        if stop_words.is_empty() {
            return Err(format!("{ENV_STOP_WORDS} must contain at least one word").into());
        }

        let bind_addr = env::var(ENV_BIND_ADDR).unwrap_or_else(|_| "127.0.0.1:8090".to_string());
        let next_webhook_url = env::var(ENV_WEBHOOK_URL)
            .map_err(|_| format!("{ENV_WEBHOOK_URL} is not set"))?
            .trim()
            .to_string();
        if next_webhook_url.is_empty() {
            return Err(format!("{ENV_WEBHOOK_URL} must not be empty").into());
        }

        Ok(Self {
            activation_words,
            stop_words,
            bind_addr,
            next_webhook_url,
        })
    }
}
