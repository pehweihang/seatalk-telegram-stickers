use std::borrow::Cow;

use async_trait::async_trait;
use http::{header, Method, Request};
use serde::de::DeserializeOwned;

use super::{
    api::error::{ApiError, BodyError},
    client::{AsyncClient, Client},
    query::{self, AsyncQuery, Query},
};

pub trait Endpoint {
    fn method(&self) -> Method;

    fn endpoint(&self) -> Cow<'static, str>;

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        Ok(None)
    }

    fn require_auth(&self) -> bool;
}

impl<E, T, C> Query<T, C> for E
where
    E: Endpoint,
    T: DeserializeOwned,
    C: Client,
{
    fn query(&self, client: &C) -> Result<T, ApiError<<C>::Error>> {
        let url = client.rest_endpoint(&self.endpoint())?;
        let req = Request::builder()
            .method(self.method())
            .uri(query::url_to_http_uri(url));

        let (req, data) = if let Some((mime, data)) = self.body()? {
            let req = req.header(header::CONTENT_TYPE, mime);
            (req, data)
        } else {
            (req, Vec::new())
        };

        let rsp = if self.require_auth() {
            client.rest(req, data)?
        } else {
            client.rest_no_auth(req, data)?
        };

        let status = rsp.status();

        let v = if let Ok(v) = serde_json::from_slice(rsp.body()) {
            v
        } else {
            return Err(ApiError::server_error(status, rsp.body()));
        };

        if !status.is_success() {
            return Err(ApiError::from_seatalk(v));
        };

        let Some(code) = v.get("code") else {
            return Err(ApiError::from_seatalk(v));
        };

        if code != 0 {
            return Err(ApiError::from_seatalk(v));
        }

        serde_json::from_value::<T>(v).map_err(ApiError::data_type::<T>)
    }
}

#[async_trait]
impl<E, T, C> AsyncQuery<T, C> for E
where
    E: Endpoint + Sync,
    T: DeserializeOwned + 'static,
    C: AsyncClient + Sync,
{
    async fn query_async(&self, client: &C) -> Result<T, ApiError<C::Error>> {
        let url = client.rest_endpoint(&self.endpoint())?;
        let req = Request::builder()
            .method(self.method())
            .uri(query::url_to_http_uri(url));

        let (req, data) = if let Some((mime, data)) = self.body()? {
            let req = req.header(header::CONTENT_TYPE, mime);
            (req, data)
        } else {
            (req, Vec::new())
        };

        let rsp = if self.require_auth() {
            client.rest_async(req, data).await?
        } else {
            client.rest_async_no_auth(req, data).await?
        };

        let status = rsp.status();

        let v = if let Ok(v) = serde_json::from_slice(rsp.body()) {
            v
        } else {
            return Err(ApiError::server_error(status, rsp.body()));
        };

        if !status.is_success() {
            return Err(ApiError::from_seatalk(v));
        };

        let Some(code) = v.get("code") else {
            return Err(ApiError::from_seatalk(v));
        };

        if code != 0 {
            return Err(ApiError::from_seatalk(v));
        }

        serde_json::from_value::<T>(v).map_err(ApiError::data_type::<T>)
    }
}

pub struct Ignore<E> {
    endpoint: E,
}

pub fn ignore<E>(endpoint: E) -> Ignore<E> {
    Ignore { endpoint }
}

impl<E, C> Query<(), C> for Ignore<E>
where
    E: Endpoint,
    C: Client,
{
    fn query(&self, client: &C) -> Result<(), ApiError<<C>::Error>> {
        let url = client.rest_endpoint(&self.endpoint.endpoint())?;
        let req = Request::builder()
            .method(self.endpoint.method())
            .uri(query::url_to_http_uri(url));

        let (req, data) = if let Some((mime, data)) = self.endpoint.body()? {
            let req = req.header(header::CONTENT_TYPE, mime);
            (req, data)
        } else {
            (req, Vec::new())
        };

        let rsp = if self.endpoint.require_auth() {
            client.rest(req, data)?
        } else {
            client.rest_no_auth(req, data)?
        };

        let status = rsp.status();

        let v = if let Ok(v) = serde_json::from_slice(rsp.body()) {
            v
        } else {
            return Err(ApiError::server_error(status, rsp.body()));
        };

        if !status.is_success() {
            return Err(ApiError::from_seatalk(v));
        };

        let Some(code) = v.get("code") else {
            return Err(ApiError::from_seatalk(v));
        };

        if code != 0 {
            return Err(ApiError::from_seatalk(v));
        }

        Ok(())
    }
}

#[async_trait]
impl<E, C> AsyncQuery<(), C> for Ignore<E>
where
    E: Endpoint + Sync,
    C: AsyncClient + Sync,
{
    async fn query_async(&self, client: &C) -> Result<(), ApiError<C::Error>> {
        let url = client.rest_endpoint(&self.endpoint.endpoint())?;
        let req = Request::builder()
            .method(self.endpoint.method())
            .uri(query::url_to_http_uri(url));

        let (req, data) = if let Some((mime, data)) = self.endpoint.body()? {
            let req = req.header(header::CONTENT_TYPE, mime);
            (req, data)
        } else {
            (req, Vec::new())
        };

        let rsp = if self.endpoint.require_auth() {
            client.rest_async(req, data).await?
        } else {
            client.rest_async_no_auth(req, data).await?
        };

        let status = rsp.status();

        let v = if let Ok(v) = serde_json::from_slice(rsp.body()) {
            v
        } else {
            return Err(ApiError::server_error(status, rsp.body()));
        };

        if !status.is_success() {
            return Err(ApiError::from_seatalk(v));
        };

        let Some(code) = v.get("code") else {
            return Err(ApiError::from_seatalk(v));
        };

        if code != 0 {
            tracing::error!("got code: {}", code);
            return Err(ApiError::from_seatalk(v));
        }

        Ok(())
    }
}
