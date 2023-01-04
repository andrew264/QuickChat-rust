use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Message{
    username: String,
    message: String,
    timestamp: i64,
}

impl Message {
    pub(crate) fn new(username: String, message: String, timestamp: Option<i64>) -> Message {
        Message {
            username,
            message,
            timestamp: timestamp.unwrap_or_else(|| {
                let now = Local::now();
                now.timestamp()
            }),
        }
    }

    pub(crate) fn new_from_bytes(bytes: &[u8]) -> Message {
        let msg = String::from_utf8_lossy(&bytes);
        let msg: Message = serde_json::from_str(&msg).unwrap();
        msg
    }

    pub(crate) fn to_string(&self) -> String {
        format!("{} @ {}: {}", self.username, self.format_timestamp(), self.message)
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8>{
        self.to_json().as_bytes().to_vec()
    }

    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub(crate) fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub(crate) fn get_username(&self) -> String {
        self.username.clone()
    }

    pub(crate) fn get_message(&self) -> String {
        self.message.clone()
    }

    pub(crate) fn format_timestamp(&self) -> String {
        let timestamp = self.get_timestamp();
        let dt = Local.timestamp_opt(timestamp, 0)
            .unwrap();
        dt.format("%I:%M:%S %p").to_string()
    }
}