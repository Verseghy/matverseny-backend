use crate::{error, Error};
use axum::{
    async_trait,
    body::HttpBody,
    extract::{rejection::JsonRejection, FromRequest},
    http::Request,
    response::{IntoResponse, Response},
    BoxError,
};
use serde::{de::DeserializeOwned, Serialize};
use validator::Validate;

pub struct Json<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for Json<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => match rejection {
                JsonRejection::JsonDataError(_) => Err(error::JSON_MISSING_FIELDS),
                JsonRejection::JsonSyntaxError(_) => Err(error::JSON_SYNTAX_ERROR),
                JsonRejection::MissingJsonContentType(_) => Err(error::JSON_CONTENT_TYPE),
                err => Err(Error::internal(err)),
            },
        }
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(json) = Json::<T>::from_request(req, state).await?;

        json.validate().map_err(|_| error::JSON_VALIDATE_INVALID)?;

        Ok(ValidatedJson(json))
    }
}
