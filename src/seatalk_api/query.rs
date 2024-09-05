use http::Uri;
use url::Url;

use crate::seatalk_api::client::{AsyncClient, Client};
use async_trait::async_trait;

use super::api::error::ApiError;

pub fn url_to_http_uri(url: Url) -> Uri {
    url.as_str()
        .parse()
        .expect("failed to parse a url::Url as an http::Uri")
}

pub trait Query<T, C>
where
    C: Client,
{
    fn query(&self, client: &C) -> Result<T, ApiError<C::Error>>;
}

#[async_trait]
pub trait AsyncQuery<T, C>
where
    C: AsyncClient,
{
    async fn query_async(&self, client: &C) -> Result<T, ApiError<C::Error>>;
}
