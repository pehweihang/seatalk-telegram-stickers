use std::borrow::Cow;

use serde::Serialize;

use crate::seatalk_api::endpoint::Endpoint;

use super::{common::MessageType, error::BodyError};

#[derive(Debug, Serialize)]
pub struct SendGroupMessage {
    group_id: String,
    message: GroupMessage,
}

#[derive(Debug, Serialize)]
#[serde(tag = "tag")]
pub enum GroupMessage {
    #[serde(rename = "text")]
    Text {
        text: TextGroupMessage,
        quoted_message_id: Option<String>,
        thread_id: Option<String>,
    },
    #[serde(rename = "image")]
    Image {
        image: ImageGroupMessage,
        quoted_message_id: Option<String>,
        thread_id: Option<String>,
    },
}

#[derive(Debug, Serialize)]
pub struct TextGroupMessage {
    format: u8,
    content: String,
}

#[derive(Debug, Serialize)]
pub struct ImageGroupMessage {
    content: String,
}

impl SendGroupMessage {
    pub fn new(
        group_id: impl Into<String>,
        thread_id: Option<String>,
        text: impl Into<String>,
        message_type: MessageType,
        quoted_message_id: Option<String>,
    ) -> Self {
        match message_type {
            MessageType::Text => {
                Self::new_text_message(group_id, thread_id, text, quoted_message_id)
            }
            MessageType::Image => {
                Self::new_image_message(group_id, thread_id, text, quoted_message_id)
            }
        }
    }
    pub fn new_text_message(
        group_id: impl Into<String>,
        thread_id: Option<String>,
        text: impl Into<String>,
        quoted_message_id: Option<String>,
    ) -> Self {
        Self {
            group_id: group_id.into(),
            message: GroupMessage::Text {
                text: TextGroupMessage {
                    format: 1,
                    content: text.into(),
                },
                quoted_message_id,
                thread_id,
            },
        }
    }
    pub fn new_image_message(
        group_id: impl Into<String>,
        thread_id: Option<String>,
        text: impl Into<String>,
        quoted_message_id: Option<String>,
    ) -> Self {
        Self {
            group_id: group_id.into(),
            message: GroupMessage::Image {
                image: ImageGroupMessage {
                    content: text.into(),
                },
                quoted_message_id,
                thread_id,
            },
        }
    }
}

impl Endpoint for SendGroupMessage {
    fn method(&self) -> http::Method {
        http::Method::POST
    }

    fn endpoint(&self) -> std::borrow::Cow<'static, str> {
        Cow::from("messaging/v2/group_chat")
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        Ok(Some((
            "application/json",
            serde_json::to_string(self)?.into_bytes(),
        )))
    }

    fn require_auth(&self) -> bool {
        true
    }
}
