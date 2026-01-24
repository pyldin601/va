use std::env;
use std::path::PathBuf;

const VOSK_MODEL_PATH_ENV: &str = "VOSK_MODEL_PATH";

pub(crate) fn get_vosk_model_path() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let path = env::var(VOSK_MODEL_PATH_ENV)
        .map(PathBuf::from)
        .map_err(|_| format!("{VOSK_MODEL_PATH_ENV} is not set"))?;
    Ok(path)
}
