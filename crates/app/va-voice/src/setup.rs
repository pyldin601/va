use crate::config::Config;
use crate::error::Error;
use std::sync::{Arc, Mutex};
use vosk::{Model, Recognizer};

pub(crate) fn setup_vosk_model(config: &Config) -> Result<Arc<Model>, Error> {
    let model = Model::new(&config.vosk_model_path).ok_or("Failed to load VOSK model")?;

    Ok(Arc::new(model))
}

pub(crate) fn setup_recognizer(
    model: &Model,
    sample_rate: f32,
) -> Result<Arc<Mutex<Recognizer>>, Error> {
    let recognizer = Arc::new(Mutex::new(
        Recognizer::new(&model, sample_rate).ok_or("Failed to create recognizer")?,
    ));

    Ok(recognizer)
}
