mod utils;

use http::header::AUTHORIZATION;
use utils::prelude::*;

#[tokio::test]
async fn no_claims() {
    let env = setup().await;

    let res = env.post("/register").json(&json!({})).send().await;

    assert_error!(res, error::COULD_NOT_GET_CLAIMS);
}

#[tokio::test]
async fn not_bearer_token() {
    let env = setup().await;

    let res = env
        .post("/register")
        .header(AUTHORIZATION, "asd")
        .json(&json!({}))
        .send()
        .await;

    assert_error!(res, error::COULD_NOT_GET_CLAIMS);
}

#[tokio::test]
async fn invalid_claims() {
    let env = setup().await;

    let res = env
        .post("/register")
        .header(AUTHORIZATION, "Bearer test.test.test")
        .json(&json!({}))
        .send()
        .await;

    assert_error!(res, error::COULD_NOT_GET_CLAIMS);
}

#[tokio::test]
#[ignore]
async fn wrong_jwt_signature() {
    // TODO: test correct jwt body but wrong signature
}

#[tokio::test]
async fn success() {
    let env = setup().await;
    let user = iam::register_user(&env).await;

    let res = env
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
async fn already_registered() {
    let env = setup().await;
    let user = iam::register_user(&env).await;

    let res = env
        .post("/register")
        .user(&user)
        .json(&json!({
            "school": "Test School",
            "class": 9,
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let res = env
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
