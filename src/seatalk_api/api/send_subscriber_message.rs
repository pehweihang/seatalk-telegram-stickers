use std::borrow::Cow;

use http::Method;
use serde::Serialize;

use crate::seatalk_api::endpoint::Endpoint;

use super::{common::MessageType, error::BodyError};

#[derive(Debug, Serialize)]
pub struct SendSubscriberMessage {
    employee_code: String,
    message: SubscriberMessage,
}

#[derive(Debug, Serialize)]
#[serde(tag = "tag")]
pub enum SubscriberMessage {
    #[serde(rename = "text")]
    Text { text: TextSubscriberMessage },
    #[serde(rename = "image")]
    Image { image: ImageSubscriberMessage },
}

#[derive(Debug, Serialize)]
pub struct TextSubscriberMessage {
    format: u8,
    content: String,
}

#[derive(Debug, Serialize)]
pub struct ImageSubscriberMessage {
    pub content: String,
}

impl SendSubscriberMessage {
    pub fn new(
        employee_code: impl Into<String>,
        message_type: MessageType,
        content: impl Into<String>,
    ) -> Self {
        match message_type {
            MessageType::Text => Self {
                employee_code: employee_code.into(),
                message: SubscriberMessage::Text {
                    text: TextSubscriberMessage {
                        format: 1,
                        content: content.into(),
                    },
                },
            },
            MessageType::Image => Self {
                employee_code: employee_code.into(),
                message: SubscriberMessage::Image {
                    image: ImageSubscriberMessage {
                        content: content.into(),
                    },
                },
            },
        }
    }
}

impl Endpoint for SendSubscriberMessage {
    fn method(&self) -> http::Method {
        Method::POST
    }

    fn endpoint(&self) -> std::borrow::Cow<'static, str> {
        Cow::from("messaging/v2/single_chat")
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
