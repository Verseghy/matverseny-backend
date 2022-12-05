mod constants;
mod db;

pub use constants::*;
pub use db::*;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::{BufMut, Bytes, BytesMut};
use sea_orm::DbErr;
use serde_json::json;

#[derive(Debug)]
pub struct ErrorInner<'a> {
    status: Option<StatusCode>,
    code: u32,
    message: &'a str,
}

pub type Error = ErrorInner<'static>;
pub type Result<T> = std::result::Result<T, Error>;

impl<'a> ErrorInner<'a> {
    #[inline]
    pub fn internal<E: Into<Box<dyn std::error::Error>>>(error: E) -> Self {
        error!("internal error: {}", error.into());
        constants::INTERNAL
    }

    #[inline]
    const fn new(status: Option<StatusCode>, code: u32, message: &'a str) -> ErrorInner<'a> {
        ErrorInner {
            status,
            code,
            message,
        }
    }

    #[inline]
    pub const fn code(&self) -> u32 {
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

impl IntoResponse for ErrorInner<'_> {
    #[inline]
    fn into_response(self) -> Response {
        let Some(status) = self.status else {
            panic!("cannot convert an error without status to a response")
        };

        (status, self.to_bytes()).into_response()
    }
}

impl From<DbErr> for ErrorInner<'_> {
    #[inline]
    fn from(error: DbErr) -> Self {
        Error::internal(error)
    }
}

impl From<axum::Error> for ErrorInner<'_> {
    #[inline]
    fn from(error: axum::Error) -> Self {
        Error::internal(error)
    }
}

macro_rules! const_error {
    (
        #[error($code:literal, $msg:literal)]
        $(#[status($status:ident)])?
        const $name:ident;
    ) => {
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

pub(self) use const_error;
