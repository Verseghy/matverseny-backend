mod constants;

pub use constants::*;

use crate::json::Json;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::DbErr;
use serde_json::json;

pub enum Error {
    Internal(Box<dyn std::error::Error>),
    Other(StatusCode, u32, &'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn internal<E: Into<Box<dyn std::error::Error>>>(error: E) -> Self {
        let error = error.into();
        tracing::error!("internal error: {}", error);
        Error::Internal(error)
    }

    pub(self) const fn new(status: StatusCode, code: u32, msg: &'static str) -> Self {
        Error::Other(status, code, msg)
    }

    pub const fn code(&self) -> u32 {
        match self {
            Error::Other(_, code, _) => *code,
            Error::Internal(_) => 0,
        }
    }

    pub const fn status(&self) -> StatusCode {
        match self {
            Error::Other(status, _, _) => *status,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        fn response_with_msg(status: StatusCode, code: u32, msg: &'static str) -> Response {
            (
                status,
                Json(json!({
                    "code": code,
                    "error": msg,
                })),
            )
                .into_response()
        }

        match self {
            Self::Internal(_) => response_with_msg(
                StatusCode::INTERNAL_SERVER_ERROR,
                0,
                "internal server error",
            ),
            Self::Other(status, code, msg) => response_with_msg(status, code, msg),
        }
    }
}

impl From<DbErr> for Error {
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
