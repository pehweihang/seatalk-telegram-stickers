use chrono::{serde::ts_seconds, DateTime, TimeDelta, Utc};
use http::{HeaderMap, HeaderValue};
use serde::Deserialize;
use thiserror::Error;

use super::{api, client::AsyncClient, query::AsyncQuery};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("header value error: {}", source)]
    HeaderVale {
        #[from]
        source: http::header::InvalidHeaderValue,
    },
    #[error("invalid token")]
    InvalidToken,
}

type AuthResult<T> = Result<T, AuthError>;

#[derive(Debug, Deserialize)]
pub struct AccessToken {
    pub app_access_token: String,
    #[serde(deserialize_with = "ts_seconds::deserialize")]
    pub expire: DateTime<Utc>,
}

impl AccessToken {
    pub fn is_expired(&self) -> bool {
        Utc::now() + TimeDelta::try_seconds(60).unwrap() >= self.expire
    }

    pub fn set_headers<'a>(
        &'a self,
        headers: &'a mut HeaderMap<HeaderValue>,
    ) -> AuthResult<&mut HeaderMap<HeaderValue>> {
        let value = format!("Bearer {}", &self.app_access_token);
        let mut header_value = HeaderValue::from_str(&value)?;
        header_value.set_sensitive(true);
        headers.insert(http::header::AUTHORIZATION, header_value);
        Ok(headers)
    }
}

#[derive(Debug)]
pub struct Auth {
    pub app_id: String,
    pub app_secret: String,
}

impl Auth {
    pub async fn get_access_token_async<C>(
        &self,
        api: &C,
    ) -> Result<AccessToken, api::ApiError<C::Error>>
    where
        C: AsyncClient + Sync,
    {
        let token: AccessToken =
            api::GetAccessToken::new(self.app_id.to_owned(), self.app_secret.to_owned())
                .query_async(api)
                .await?;
        Ok(token)
    }
    pub async fn check_auth_async<C>(&self, api: &C) -> Result<(), api::ApiError<C::Error>>
    where
        C: AsyncClient + Sync,
    {
        // TODO
        unimplemented!()
    }
}
