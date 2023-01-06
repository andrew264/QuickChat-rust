pub(crate) enum MessageType {
    Ping,
    Join,
    Leave,
    SetUsername,
    UsernameTaken,
    UsernameAvailable,
    Message,
}

impl MessageType {
    pub(crate) fn as_int(&self) -> i32 {
        match self {
            MessageType::Ping => { 0 }
            MessageType::Join => { 1 }
            MessageType::Leave => { 2 }
            MessageType::SetUsername => { 3 }
            MessageType::UsernameTaken => { 4 }
            MessageType::UsernameAvailable => { 5 }
            MessageType::Message => { 32 }
        }
    }
    pub(crate) fn from_int(i: i32) -> MessageType {
        match i {
            0 => { MessageType::Ping }
            1 => { MessageType::Join }
            2 => { MessageType::Leave }
            3 => { MessageType::SetUsername }
            4 => { MessageType::UsernameTaken }
            5 => { MessageType::UsernameAvailable }
            32 => { MessageType::Message }
            _ => { MessageType::Message }
        }
    }

    fn as_str(&self) -> String {
        match self {
            MessageType::Ping => { "Ping".to_string() }
            MessageType::Join => { "Join".to_string() }
            MessageType::Leave => { "Leave".to_string() }
            MessageType::SetUsername => { "SetUsername".to_string() }
            MessageType::UsernameTaken => { "UsernameTaken".to_string() }
            MessageType::UsernameAvailable => { "UsernameAvailable".to_string() }
            MessageType::Message => { "Message".to_string() }
        }
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}