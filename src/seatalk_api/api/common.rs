use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Code {
    pub code: u32,
}

#[derive(Debug, Deserialize)]
pub struct MessageCode {
    pub code: u32,
    pub message_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "image")]
    Image,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Text => "text".to_string(),
                Self::Image => "image".to_string(),
            }
        )
    }
}
