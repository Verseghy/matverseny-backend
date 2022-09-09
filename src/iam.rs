use jsonwebtoken::{errors::Error, Algorithm, DecodingKey, Validation};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Claims {
    #[serde(rename = "iss")]
    pub issuer: String,
    #[serde(rename = "sub")]
    pub subject: String,
    #[serde(rename = "aud")]
    pub audience: Vec<String>,
    #[serde(rename = "exp")]
    pub expires_at: i64,
    #[serde(rename = "nbf")]
    pub not_before: i64,
    #[serde(rename = "iat")]
    pub issued_at: i64,
}

pub trait IamTrait {
    fn get_claims(&self, token: &str) -> Result<Claims, Error>;
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

static VALIDATION: Lazy<Validation> = Lazy::new(|| {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&["https://verseghy-gimnazium.net"]);
    validation.leeway = 5;

    validation
});

impl IamTrait for Iam {
    fn get_claims(&self, token: &str) -> Result<Claims, Error> {
        match jsonwebtoken::decode(token, &self.decoding, &*VALIDATION) {
            Ok(decode) => Ok(decode.claims),
            Err(err) => {
                tracing::error!("jwt error: {:?}", err);
                Err(err)
            }
        }
    }
}