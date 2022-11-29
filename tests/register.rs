mod utils;

use http::header::AUTHORIZATION;
use utils::prelude::*;

#[tokio::test]
#[serial]
async fn no_claims() {
    let app = App::new().await;

    let res = app.post("/register").json(&json!({})).send().await;

    assert_error!(res, error::COULD_NOT_GET_CLAIMS);
}

#[tokio::test]
#[serial]
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
#[serial]
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
#[serial]
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

#[tokio::test]
#[serial]
async fn already_registered() {
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

    let res = app
        .post("/register")
        .user(&user)
        .json(&json!({
            "school": "Test School",
            "class": 9,
        }))
        .send()
        .await;

    assert_error!(res, error::USER_ALREADY_EXISTS);
}
