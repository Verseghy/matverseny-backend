use crate::error::{self, Error};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use libiam::jwt::Claims;
use std::ops::Deref;
use uuid::Uuid;

pub struct UserID(Uuid);

impl UserID {
    pub fn parse_str(string: &str) -> error::Result<Self> {
        let Some(user_id) = string.strip_prefix("UserID-") else {
            return Err(error::COULD_NOT_GET_CLAIMS);
        };

        let Ok(user_id) = Uuid::parse_str(user_id) else {
            return Err(error::COULD_NOT_GET_CLAIMS);
        };

        Ok(UserID(user_id))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserID
where
    S: Send + Sync,
{
    type Rejection = Error<'static>;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Some(claims) = parts.extensions.remove::<Claims>() else {
            return Err(error::COULD_NOT_GET_CLAIMS);
        };

        UserID::parse_str(&claims.sub)
    }
}

impl Deref for UserID {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
