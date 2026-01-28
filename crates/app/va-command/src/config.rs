use std::env;

use crate::error::Error;

const ENV_BIND_ADDR: &str = "BIND_ADDR";
const ENV_OLLAMA_BASE_URL: &str = "OLLAMA_BASE_URL";
const ENV_OLLAMA_MODEL: &str = "OLLAMA_MODEL";

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8092";
const DEFAULT_OLLAMA_BASE_URL: &str = "http://localhost:11434";
const DEFAULT_OLLAMA_MODEL: &str = "gemma3n";

#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) bind_addr: String,
    pub(crate) ollama_base_url: String,
    pub(crate) ollama_model: String,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self, Error> {
        let bind_addr = env::var(ENV_BIND_ADDR).unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
        let ollama_base_url = env::var(ENV_OLLAMA_BASE_URL)
            .unwrap_or_else(|_| DEFAULT_OLLAMA_BASE_URL.to_string())
            .trim()
            .to_string();
        if ollama_base_url.is_empty() {
            return Err(format!("{ENV_OLLAMA_BASE_URL} must not be empty").into());
        }

        let ollama_model = env::var(ENV_OLLAMA_MODEL)
            .unwrap_or_else(|_| DEFAULT_OLLAMA_MODEL.to_string())
            .trim()
            .to_string();
        if ollama_model.is_empty() {
            return Err(format!("{ENV_OLLAMA_MODEL} must not be empty").into());
        }

        Ok(Self {
            bind_addr,
            ollama_base_url,
            ollama_model,
        })
    }
}
