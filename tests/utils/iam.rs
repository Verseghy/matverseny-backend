use super::UserLike;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::{
    env,
    sync::atomic::{AtomicU64, Ordering},
};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub access_token: String,
}

impl UserLike for User {
    fn access_token(&self) -> &String {
        &self.access_token
    }
}

static TEST_ID: Lazy<String> = Lazy::new(|| {
    let id = Uuid::new_v4();
    id.as_hyphenated()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
});

static USER_COUNT: AtomicU64 = AtomicU64::new(0);
static USER_PASSWORD: &str = "test";

fn get_url(url: &str) -> String {
    format!("{}{}", env::var("IAM_URL").expect("IAM_URL not set"), url)
}

pub async fn register_user() -> User {
    let email = format!(
        "TestUser-{}-{}@test.test",
        *TEST_ID,
        USER_COUNT.fetch_add(1, Ordering::Relaxed)
    );

    let response = Client::new()
        .post(get_url("/v1/users/register"))
        .json(&json!({
            "name": "Test User",
            "email": email,
            "password": USER_PASSWORD,
        }))
        .send()
        .await
        .expect("registration failed")
        .json::<serde_json::Value>()
        .await
        .expect("failed to deserialize");

    let id = response["id"].as_str().expect("not string").to_owned();

    let response = Client::new()
        .post(get_url("/v1/users/login"))
        .json(&json!({
            "email": email,
            "password": USER_PASSWORD,
        }))
        .send()
        .await
        .expect("failed to send")
        .json::<serde_json::Value>()
        .await
        .expect("failed to deserialize");

    let access_token = response["token"].as_str().expect("not string").to_owned();

    User {
        id,
        email,
        access_token,
    }
}
