use chrono::{Local, TimeZone};
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

    pub(crate) fn from_bytes(bytes: &[u8]) -> Vec<Message> {
        let str_msg = String::from_utf8_lossy(&bytes);
        trace!("Received messages: {}", str_msg);
        let v: Value = serde_json::from_str(&str_msg).unwrap();
        let mut messages = Vec::new();
        if let Value::Array(msg_array) = v {
            for msg in msg_array {
                trace!("Received message: {}", msg);
                messages.push(serde_json::from_str::<Message>(&*msg.to_string()).unwrap())
            }
        }
        debug!("Received {} messages", messages.len());
        messages
    }

    pub(crate) fn to_string(&self) -> String {
        match MessageType::from_int(self.type_) {
            MessageType::Ping => {
                format!("Ping: {}",
                        (Local::now() - Local.timestamp_nanos(self.timestamp)).num_milliseconds())
            }
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
                    self.username
                )
            }
            MessageType::UsernameTaken => {
                format!(
                    "[SERVER]: {} is not available",
                    self.username
                )
            }
            _ => {
                format!("{}", MessageType::from_int(self.type_))
            }
        }
    }

    fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub(crate) fn get_username(&self) -> String {
        self.username.clone()
    }

    pub(crate) fn format_timestamp(&self) -> String {
        let timestamp = self.get_timestamp();
        let dt = Local.timestamp_nanos(timestamp);
        dt.format("%I:%M:%S %p").to_string()
    }

    pub(crate) fn get_type(&self) -> MessageType {
        MessageType::from_int(self.type_)
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Message {
            username: self.username.clone(),
            message: self.message.clone(),
            timestamp: self.timestamp,
            type_: self.type_,
        }
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
                now.timestamp_nanos()
            },
            type_: self.type_.as_int(),
        }
    }
}