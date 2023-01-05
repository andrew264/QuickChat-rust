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
    pub(crate) fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Message {
        let msg = String::from_utf8_lossy(&bytes);
        let msg: Message = serde_json::from_str(&msg).unwrap();
        msg
    }

    pub(crate) fn to_string(&self) -> String {
        match MessageType::from_int(self.type_) {
            MessageType::Message => {
                format!(
                    "[{} @ {}]: {}",
                    self.format_timestamp(),
                    self.username,
                    self.message
                )
            }
            MessageType::Join => {
                format!(
                    "[SERVER]: {} joined the chat",
                    self.username
                )
            }
            MessageType::Leave => {
                format!(
                    "[SERVER]: {} left the chat",
                    self.username
                )
            }
            MessageType::SetUsername => {
                format!(
                    "[SERVER]: username to {}",
                    self.username,
                )
            }
            MessageType::UsernameAvailable => {
                format!(
                    "[SERVER]: {} is available",
                    self.message
                )
            }
            MessageType::UsernameTaken => {
                format!(
                    "[SERVER]: {} is not available",
                    self.message
                )
            }
        }
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        self.to_json().as_bytes().to_vec()
    }

    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn get_timestamp(&self) -> i64 {
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
}

pub(crate) struct MessageBuilder {
    username: String,
    message: String,
    type_: MessageType,
}

impl MessageBuilder {
    pub(crate) fn new() -> MessageBuilder {
        MessageBuilder {
            username: String::new(),
            message: String::new(),
            type_: MessageType::Message,
        }
    }

    pub(crate) fn username(&mut self, username: &String) -> &mut MessageBuilder {
        self.username = username.clone();
        self
    }

    pub(crate) fn message(&mut self, message: &String) -> &mut MessageBuilder {
        self.message = message.clone();
        self
    }

    pub(crate) fn message_type(&mut self, type_: MessageType) -> &mut MessageBuilder {
        self.type_ = type_;
        self
    }

    pub(crate) fn build(&self) -> Message {
        Message {
            username: self.username.clone(),
            message: self.message.clone(),
            timestamp: {
                let now = Local::now();
                now.timestamp()
            },
            type_: self.type_.as_int(),
        }
    }
}