use governor::{DefaultDirectRateLimiter, Jitter, Quota, RateLimiter};
use std::{num::NonZeroU32, sync::Arc};
use tokio::sync::Mutex;

use async_trait::async_trait;
use bytes::Bytes;
use http::Response;
use thiserror::Error;
use url::Url;

use super::{
    api,
    auth::{AccessToken, Auth, AuthError},
    client::{self, RestClient},
};
use reqwest::Client as AsyncClient;

#[derive(Debug, Error)]
pub enum SeatalkError {
    #[error("failed to parse url: {}", source)]
    UrlParse {
        #[from]
        source: url::ParseError,
    },
    #[error("api error: {}", source)]
    Api {
        #[from]
        source: api::ApiError<RestError>,
    },
}

pub type SeatalkResult<T> = Result<T, SeatalkError>;

#[derive(Debug)]
pub struct AsyncSeatalk {
    client: reqwest::Client,
    lim: DefaultDirectRateLimiter,
    rest_url: Url,
    auth: Auth,
    access_token: Arc<Mutex<Option<AccessToken>>>,
}

impl AsyncSeatalk {
    pub async fn new(protocol: &str, host: &str, auth: Auth) -> SeatalkResult<Self> {
        let rest_url = Url::parse(&format!("{}://{}/", protocol, host))?;
        let access_token = Arc::new(Mutex::new(None));
        let client = AsyncClient::new();

        let lim =
            RateLimiter::direct(Quota::with_period(std::time::Duration::from_secs(2)).unwrap());

        let api = Self {
            client,
            lim,
            rest_url,
            auth,
            access_token,
        };

        *api.access_token.lock().await = Some(api.auth.get_access_token_async(&api).await?);

        Ok(api)
    }
}

#[derive(Debug, Error)]
pub enum RestError {
    #[error("error setting auth header: {}", source)]
    AuthError {
        #[from]
        source: AuthError,
    },
    #[error("communication with seatalk: {}", source)]
    Communication {
        #[from]
        source: reqwest::Error,
    },
    #[error("`http` error: {}", source)]
    Http {
        #[from]
        source: http::Error,
    },
}

impl RestClient for AsyncSeatalk {
    type Error = RestError;
    fn rest_endpoint(&self, endpoint: &str) -> Result<Url, api::ApiError<Self::Error>> {
        Ok(self.rest_url.join(endpoint)?)
    }
}

#[async_trait]
impl client::AsyncClient for AsyncSeatalk {
    async fn rest_async(
        &self,
        mut request: http::request::Builder,
        body: Vec<u8>,
    ) -> Result<Response<Bytes>, api::ApiError<<Self as RestClient>::Error>> {
        use futures_util::TryFutureExt;
        let should_reauth = self
            .access_token
            .lock()
            .await
            .as_ref()
            .map_or(true, |t| t.is_expired());

        if should_reauth {
            *self.access_token.lock().await = Some(self.auth.get_access_token_async(self).await?);
        }
        let call = || async {
            self.access_token
                .lock()
                .await
                .as_ref()
                .ok_or(AuthError::InvalidToken)?
                .set_headers(request.headers_mut().unwrap())?;
            let http_request = request.body(body)?;
            let request = http_request.try_into()?;
            self.lim.until_ready().await;
            let rsp = self.client.execute(request).await?;

            let mut http_rsp = Response::builder()
                .status(rsp.status())
                .version(rsp.version());
            let headers = http_rsp.headers_mut().unwrap();
            for (key, value) in rsp.headers() {
                headers.insert(key, value.clone());
            }
            Ok(http_rsp.body(rsp.bytes().await?)?)
        };
        call().map_err(api::ApiError::client).await
    }

    async fn rest_async_no_auth(
        &self,
        request: http::request::Builder,
        body: Vec<u8>,
    ) -> Result<Response<Bytes>, api::ApiError<<Self as RestClient>::Error>> {
        use futures_util::TryFutureExt;
        let call = || async {
            let http_request = request.body(body)?;
            let request = http_request.try_into()?;
            self.lim.until_ready().await;
            let rsp = self.client.execute(request).await?;

            let mut http_rsp = Response::builder()
                .status(rsp.status())
                .version(rsp.version());
            let headers = http_rsp.headers_mut().unwrap();
            for (key, value) in rsp.headers() {
                headers.insert(key, value.clone());
            }
            Ok(http_rsp.body(rsp.bytes().await?)?)
        };
        call().map_err(api::ApiError::client).await
    }
}
