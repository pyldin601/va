use std::env;

const VOSK_MODEL_PATH_ENV: &str = "VOSK_MODEL_PATH";
const BIND_ADDR_ENV: &str = "BIND_ADDR";

pub(crate) fn get_vosk_model_path() -> String {
    env::var(VOSK_MODEL_PATH_ENV).expect(&format!("{VOSK_MODEL_PATH_ENV} is not set"))
}

pub(crate) fn get_bind_addr() -> String {
    env::var(BIND_ADDR_ENV).unwrap_or_else(|_| "127.0.0.1:8080".to_string())
}
