use crate::{
    error::{self, Result},
};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use once_cell::sync::Lazy;
use serde::{de, Deserialize, Deserializer};
use std::env;
use uuid::Uuid;

fn deserialize_subject<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Uuid;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a uuid prefixed with `UserID-`")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if let Some(id) = v.strip_prefix("UserID-") {
                if let Ok(id) = Uuid::parse_str(id) {
                    Ok(id)
                } else {
                    Err(de::Error::invalid_value(de::Unexpected::Str(id), &self))
                }
            } else {
                Err(de::Error::invalid_value(de::Unexpected::Str(v), &self))
            }
        }
    }

    deserializer.deserialize_str(Visitor)
}

#[derive(Debug, Deserialize)]
pub struct Claims {
    #[serde(rename = "iss")]
    pub issuer: String,
    #[serde(rename = "sub", deserialize_with = "deserialize_subject")]
    pub subject: Uuid,
    #[serde(rename = "aud")]
    pub audience: Vec<String>,
    #[serde(rename = "exp")]
    pub expires_at: i64,
    #[serde(rename = "nbf")]
    pub not_before: i64,
    #[serde(rename = "iat")]
    pub issued_at: i64,
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = error::Error;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        parts
            .extensions
            .remove::<Claims>()
            .ok_or(error::COULD_NOT_GET_CLAIMS)
    }
}

pub trait IamTrait {
    fn get_claims(&self, token: &str) -> Result<Claims>;
}

pub struct Iam {
    decoding: DecodingKey,
}

impl Iam {
    pub fn new() -> Self {
        Self {
            decoding: DecodingKey::from_rsa_pem(
                env::var("IAM_JWT_RSA_PUBLIC")
                    .expect("IAM_JWT_RSA_PUBLIC not set")
                    .as_ref(),
            )
            .expect("IAM_JWT_RSA_PUBLIC invlida"),
        }
    }
}

impl Default for Iam {
    fn default() -> Self {
        Self::new()
    }
}

static VALIDATION: Lazy<Validation> = Lazy::new(|| {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&["https://verseghy-gimnazium.net"]);
    validation.leeway = 5;

    validation
});

impl IamTrait for Iam {
    fn get_claims(&self, token: &str) -> Result<Claims> {
        match jsonwebtoken::decode(token, &self.decoding, &VALIDATION) {
            Ok(decode) => Ok(decode.claims),
            Err(error) => {
                warn!(token, error = error.to_string(), "tried invalid token");
                Err(error::JWT_INVALID_TOKEN)
            }
        }
    }
}
