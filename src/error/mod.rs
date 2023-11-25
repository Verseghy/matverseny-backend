mod constants;
mod db;

pub use constants::*;
pub use db::*;

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::{BufMut, Bytes, BytesMut};
use rdkafka::{error::KafkaError, message::OwnedMessage};
use sea_orm::DbErr;
use serde_json::json;

#[derive(Debug)]
pub struct Error<'a> {
    status: Option<StatusCode>,
    code: &'static str,
    message: &'a str,
}

pub type Result<T = ()> = std::result::Result<T, Error<'static>>;

impl<'a> Error<'a> {
    // #[inline]
    // pub fn internal<E: Into<Box<dyn std::error::Error>>>(error: E) -> Self {
    //     error!("internal error: {}", error.into());
    //     constants::INTERNAL
    // }

    #[inline]
    const fn new(status: Option<StatusCode>, code: &'static str, message: &'a str) -> Error<'a> {
        Self {
            status,
            code,
            message,
        }
    }

    #[inline]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    #[inline]
    pub const fn status(&self) -> Option<StatusCode> {
        self.status
    }

    #[inline]
    pub const fn message(&self) -> &str {
        self.message
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(128).writer();

        serde_json::to_writer(
            &mut buf,
            &json!({
                "code": self.code(),
                "error": self.message(),
            }),
        )
        .expect("failed to serialize error");

        buf.into_inner().freeze()
    }
}

impl IntoResponse for Error<'_> {
    #[inline]
    fn into_response(self) -> Response {
        let Some(status) = self.status else {
            panic!("cannot convert an error without status to a response")
        };

        let buf = self.to_bytes();
        let mut res = (status, buf).into_response();

        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );

        res
    }
}

impl From<DbErr> for Error<'_> {
    #[inline]
    fn from(error: DbErr) -> Self {
        error!("database error: {:?}", error);
        constants::DATABASE_ERROR
    }
}

impl From<serde_json::Error> for Error<'_> {
    #[inline]
    fn from(error: serde_json::Error) -> Self {
        error!("failed to deserialize json: {:?}", error);
        constants::JSON_DESERIALIZE
    }
}

impl From<KafkaError> for Error<'_> {
    #[inline]
    fn from(error: KafkaError) -> Self {
        error!("kafka error: {:?}", error);
        constants::KAFKA_ERROR
    }
}

impl From<(KafkaError, OwnedMessage)> for Error<'_> {
    #[inline]
    fn from((error, _): (KafkaError, OwnedMessage)) -> Self {
        error!("kafka error: {:?}", error);
        constants::KAFKA_ERROR
    }
}

macro_rules! const_error {
    (
        #[error($code:literal, $msg:literal)]
        $(#[status($status:ident)])?
        const $name:ident;
    ) => {
        macros::error_code_to_ident!($code);
        pub const $name: $crate::error::Error =
            $crate::error::Error::new($crate::error::const_error!(@status $($status)?), $code, $msg);
    };
    (@status $status:ident) => {
        Some(::axum::http::StatusCode::$status)
    };
    (@status) => {
        None
    };
}

#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use const_error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_response_has_json_content_type() {
        let error = Error::new(Some(StatusCode::OK), "", "");
        let response = error.into_response();
        let content_type = response.headers().get(http::header::CONTENT_TYPE);

        assert!(content_type.is_some(), "response");
        assert_eq!(content_type.unwrap(), "application/json");
    }
}
