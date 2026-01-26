use std::collections::HashSet;
use std::env;

use crate::error::Error;

const ENV_ACTIVATION_WORD: &str = "ACTIVATION_WORD";
const ENV_STOP_WORDS: &str = "STOP_WORDS";
const ENV_BIND_ADDR: &str = "BIND_ADDR";

#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) activation_word: String,
    pub(crate) stop_words: HashSet<String>,
    pub(crate) bind_addr: String,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self, Error> {
        let activation_word = env::var(ENV_ACTIVATION_WORD)
            .map_err(|_| format!("{ENV_ACTIVATION_WORD} is not set"))?
            .trim()
            .to_lowercase();
        if activation_word.is_empty() {
            return Err(format!("{ENV_ACTIVATION_WORD} must not be empty").into());
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

        Ok(Self {
            activation_word,
            stop_words,
            bind_addr,
        })
    }
}
