use std::{any, error::Error};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BodyError {
    #[error("Failed to serialize to JSON: {}", source)]
    SerdeJson {
        #[from]
        source: serde_json::Error,
    },
}

#[derive(Error, Debug)]
pub enum ApiError<E>
where
    E: Error + Send + Sync + 'static,
{
    #[error("client error: {}", source)]
    Client { source: E },

    #[error("failed to parse url: {}", source)]
    UrlParse {
        #[from]
        source: url::ParseError,
    },
    /// Body data could not be created.
    #[error("failed to create request body: {}", source)]
    Body {
        /// The source of the error.
        #[from]
        source: BodyError,
    },
    /// JSON deserialization from Traduora failed.
    #[error("could not parse JSON response: {}", source)]
    Json {
        /// The source of the error.
        #[from]
        source: serde_json::Error,
    },
    /// Failed to parse an expected data type from JSON.
    #[error("could not parse {} data from JSON: {}", typename, source)]
    DataType {
        /// The source of the error.
        source: serde_json::Error,
        /// The name of the type that could not be deserialized.
        typename: &'static str,
    },
    #[error("seatalk server error: {}", msg)]
    Seatalk { msg: String },
    #[error("seatalk server error {:?}", obj)]
    SeatalkObject { obj: serde_json::Value },
    #[error("seatalk internal server error {}", status)]
    SeatalkService {
        status: http::StatusCode,
        data: Vec<u8>,
    },
    #[error("seatalk server error: {:?}", obj)]
    SeatalkUnrecognized { obj: serde_json::Value },
}

impl<E> ApiError<E>
where
    E: Error + Send + Sync + 'static,
{
    pub fn client(source: E) -> Self {
        Self::Client { source }
    }

    pub(crate) fn server_error(status: http::StatusCode, body: &bytes::Bytes) -> Self {
        Self::SeatalkService {
            status,
            data: body.into_iter().copied().collect(),
        }
    }

    pub(crate) fn data_type<T>(source: serde_json::Error) -> Self {
        Self::DataType {
            source,
            typename: any::type_name::<T>(),
        }
    }

    pub(crate) fn from_seatalk(value: serde_json::Value) -> Self {
        let error_value = value.pointer("/message");

        if let Some(error_value) = error_value {
            if let Some(msg) = error_value.as_str() {
                ApiError::Seatalk { msg: msg.into() }
            } else {
                ApiError::SeatalkObject {
                    obj: error_value.clone(),
                }
            }
        } else {
            ApiError::SeatalkUnrecognized { obj: value }
        }
    }
}
