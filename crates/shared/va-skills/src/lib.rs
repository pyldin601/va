use serde::{Deserialize, Serialize};

pub const COMMAND_TIME_NOW: &str = "time.now";
pub const COMMAND_DATE_NOW: &str = "date.now";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "command")]
pub enum CommandRequest {
    #[serde(rename = "time.now")]
    TimeNow,
    #[serde(rename = "date.now")]
    DateNow,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExecuteRequest {
    #[serde(flatten)]
    pub command: CommandRequest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimeNowResult {
    pub time: String,
    pub rfc3339: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DateNowResult {
    pub date: String,
    pub rfc3339: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "command", content = "result")]
pub enum CommandResponse {
    #[serde(rename = "time.now")]
    TimeNow(TimeNowResult),
    #[serde(rename = "date.now")]
    DateNow(DateNowResult),
}
