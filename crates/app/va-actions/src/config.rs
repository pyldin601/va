use std::env;

use crate::error::Error;

const ENV_BIND_ADDR: &str = "BIND_ADDR";
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8093";

#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) bind_addr: String,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self, Error> {
        let bind_addr = env::var(ENV_BIND_ADDR).unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
        Ok(Self { bind_addr })
    }
}
