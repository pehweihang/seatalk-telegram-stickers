use std::error::Error;

use async_trait::async_trait;
use bytes::Bytes;
use http::Response;
use url::Url;

use super::api::error::ApiError;

pub trait RestClient {
    type Error: Error + Send + Sync + 'static;

    fn rest_endpoint(&self, endpoint: &str) -> Result<Url, ApiError<Self::Error>>;
}

pub trait Client: RestClient {
    fn rest(
        &self,
        request: http::request::Builder,
        body: Vec<u8>,
    ) -> Result<Response<Bytes>, ApiError<Self::Error>>;

    fn rest_no_auth(
        &self,
        request: http::request::Builder,
        body: Vec<u8>,
    ) -> Result<Response<Bytes>, ApiError<Self::Error>>;
}

#[async_trait]
pub trait AsyncClient: RestClient {
    async fn rest_async(
        &self,
        request: http::request::Builder,
        body: Vec<u8>,
    ) -> Result<Response<Bytes>, ApiError<Self::Error>>;

    async fn rest_async_no_auth(
        &self,
        request: http::request::Builder,
        body: Vec<u8>,
    ) -> Result<Response<Bytes>, ApiError<Self::Error>>;
}
