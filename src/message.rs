use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};

use crate::message_types::MessageType;

#[derive(Serialize, Deserialize)]
pub(crate) struct Message {
    username: String,
    message: String,
    timestamp: i64,
    type_: i32,
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
            type_: MessageType::Message.as_int(),
        }
    }

    pub(crate) fn new_from_bytes(bytes: &[u8]) -> Message {
        let msg = String::from_utf8_lossy(&bytes);
        let msg: Message = serde_json::from_str(&msg).unwrap();
        msg
    }

    pub(crate) fn new_from_type(type_: MessageType) -> Message {
        Message {
            username: "".to_string(),
            message: "".to_string(),
            timestamp: {
                let now = Local::now();
                now.timestamp()
            },
            type_: type_.as_int(),
        }
    }

    pub(crate) fn to_string(&self) -> String {
        format!("{} @ {}: {}", self.username, self.format_timestamp(), self.message)
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
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

    pub(crate) fn format_timestamp(&self) -> String {
        let timestamp = self.get_timestamp();
        let dt = Local.timestamp_opt(timestamp, 0)
            .unwrap();
        dt.format("%I:%M:%S %p").to_string()
    }

    pub(crate) fn get_type(&self) -> MessageType {
        MessageType::from_int(self.type_)
    }

    pub(crate) fn set_type(&mut self, type_: MessageType) {
        self.type_ = type_.as_int();
    }
}
