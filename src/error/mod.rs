mod constants;
mod db;

pub use constants::*;
pub use db::*;

use crate::json::Json;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::DbErr;
use serde_json::json;

#[derive(Debug)]
pub struct Error {
    status: StatusCode,
    code: u32,
    message: &'static str,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    #[inline]
    pub fn internal<E: Into<Box<dyn std::error::Error>>>(error: E) -> Self {
        tracing::error!("internal error: {}", error.into());
        constants::INTERNAL
    }

    #[inline]
    const fn new(status: StatusCode, code: u32, message: &'static str) -> Self {
        Error {
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
    pub const fn status(&self) -> StatusCode {
        self.status
    }
}

impl IntoResponse for Error {
    #[inline]
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({
                "code": self.code,
                "error": self.message,
            })),
        )
            .into_response()
    }
}

impl From<DbErr> for Error {
    #[inline]
    fn from(error: DbErr) -> Self {
        Error::internal(error)
    }
}

macro_rules! error {
    ($name:ident, $status:ident, $code:literal, $msg:literal) => {
        pub const $name: $crate::error::Error =
            $crate::error::Error::new(::axum::http::StatusCode::$status, $code, $msg);
    };
}

pub(self) use error;
