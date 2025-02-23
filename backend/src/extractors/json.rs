use crate::{Error, error};
use axum::{
    extract::{FromRequest, Request, rejection::JsonRejection},
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use validator::Validate;

pub struct Json<T>(pub T);

impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Error<'static>;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        #[allow(clippy::disallowed_types)]
        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => match rejection {
                JsonRejection::JsonDataError(_) => Err(error::JSON_MISSING_FIELDS),
                JsonRejection::JsonSyntaxError(_) => Err(error::JSON_SYNTAX_ERROR),
                JsonRejection::MissingJsonContentType(_) => Err(error::JSON_CONTENT_TYPE),
                // FIXME: maybe better error?
                _ => Err(error::INTERNAL),
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

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Error<'static>;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(json) = Json::<T>::from_request(req, state).await?;

        json.validate().map_err(|_| error::JSON_VALIDATE_INVALID)?;

        Ok(ValidatedJson(json))
    }
}
