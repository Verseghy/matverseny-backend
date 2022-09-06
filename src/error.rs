use crate::json::Json;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::DbErr;
use serde_json::json;

pub enum Error {
    Internal(Box<dyn std::error::Error>),
    NotFound(Option<String>),
    BadRequest(Option<String>),
    Unauthorized(Option<String>),
    Forbidden(Option<String>),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn internal<E: Into<Box<dyn std::error::Error>>>(error: E) -> Self {
        let error = error.into();
        tracing::error!("internal error: {}", error);
        Error::Internal(error)
    }

    #[allow(unused)]
    pub fn not_found<S: AsRef<str>>(msg: S) -> Self {
        Error::NotFound(Some(msg.as_ref().to_string()))
    }

    pub fn bad_request<S: AsRef<str>>(msg: S) -> Self {
        Error::BadRequest(Some(msg.as_ref().to_string()))
    }

    #[allow(unused)]
    pub fn unauthorized<S: AsRef<str>>(msg: S) -> Self {
        Error::Unauthorized(Some(msg.as_ref().to_string()))
    }

    #[allow(unused)]
    pub fn forbidden<S: AsRef<str>>(msg: S) -> Self {
        Error::Forbidden(Some(msg.as_ref().to_string()))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        fn response_with_msg<S: AsRef<str>>(status: StatusCode, msg: Option<S>) -> Response {
            if let Some(msg) = msg {
                (
                    status,
                    Json(json!({
                        "error": msg.as_ref(),
                    })),
                )
                    .into_response()
            } else {
                status.into_response()
            }
        }

        match self {
            Self::Internal(_) => response_with_msg(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some("internal server error"),
            ),
            Self::NotFound(msg) => response_with_msg(StatusCode::NOT_FOUND, msg),
            Self::BadRequest(msg) => response_with_msg(StatusCode::BAD_REQUEST, msg),
            Self::Unauthorized(msg) => response_with_msg(StatusCode::UNAUTHORIZED, msg),
            Self::Forbidden(msg) => response_with_msg(StatusCode::FORBIDDEN, msg),
        }
    }
}

impl From<DbErr> for Error {
    fn from(error: DbErr) -> Self {
        Error::internal(error)
    }
}
