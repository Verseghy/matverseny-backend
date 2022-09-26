mod utils;

use http::header::AUTHORIZATION;
use matverseny_backend::error;
use serde_json::json;
use utils::{assert_error, App};
use http::StatusCode;

#[tokio::test]
async fn no_claims() {
    let app = App::new().await;

    let res = app.post("/register").json(&json!({})).send().await;

    assert_error!(res, error::COULD_NOT_GET_CLAIMS);
}

#[tokio::test]
async fn not_bearer_token() {
    let app = App::new().await;

    let res = app
        .post("/register")
        .header(AUTHORIZATION, "asd")
        .json(&json!({}))
        .send()
        .await;

    assert_error!(res, error::COULD_NOT_GET_CLAIMS);
}

#[tokio::test]
async fn invalid_claims() {
    let app = App::new().await;

    let res = app
        .post("/register")
        .header(AUTHORIZATION, "Bearer test.test.test")
        .json(&json!({}))
        .send()
        .await;

    assert_error!(res, error::COULD_NOT_GET_CLAIMS);
}

// TODO: test correct jwt body but wrong signature

#[tokio::test]
async fn success() {
    let app = App::new().await;

    let user = utils::iam::register_user().await;

    let res = app
        .post("/register")
        .user(&user)
        .json(&json!({
            "school": "Test School",
            "class": 9,
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);
}
