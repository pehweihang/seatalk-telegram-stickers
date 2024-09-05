use std::borrow::Cow;

use http::Method;
use serde::Serialize;

use crate::seatalk_api::endpoint::Endpoint;

use super::error::BodyError;

#[derive(Debug, Serialize)]
pub struct GetAccessToken {
    app_id: String,
    app_secret: String,
}
impl GetAccessToken {
    pub fn new(app_id: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            app_secret: app_secret.into(),
        }
    }
}

impl Endpoint for GetAccessToken {
    fn method(&self) -> http::Method {
        Method::POST
    }

    fn endpoint(&self) -> std::borrow::Cow<'static, str> {
        Cow::from("auth/app_access_token")
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        Ok(Some((
            "application/json",
            serde_json::to_string(self)?.into_bytes(),
        )))
    }

    fn require_auth(&self) -> bool {
        false
    }
}
