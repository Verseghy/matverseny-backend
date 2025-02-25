use test_utils::prelude::*;

#[tokio::test]
async fn no_claims() {
    let app = get_cached_app().await;

    let res = app.post("/v1/register").json(&json!({})).send().await;

    assert_error!(res, error::COULD_NOT_GET_CLAIMS);
}

#[tokio::test]
async fn not_bearer_token() {
    let app = get_cached_app().await;

    let res = app
        .post("/v1/register")
        .header(header::AUTHORIZATION, "asd")
        .json(&json!({}))
        .send()
        .await;

    assert_error!(res, error::COULD_NOT_GET_CLAIMS);
}

#[tokio::test]
async fn invalid_claims() {
    let app = get_cached_app().await;

    let res = app
        .post("/v1/register")
        .header(header::AUTHORIZATION, "Bearer test.test.test")
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
    let app = get_cached_app().await;
    let user = iam::register_user().await;

    let res = app
        .post("/v1/register")
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
    let app = get_cached_app().await;
    let user = iam::register_user().await;

    let res = app
        .post("/v1/register")
        .user(&user)
        .json(&json!({
            "school": "Test School",
            "class": 9,
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let res = app
        .post("/v1/register")
        .user(&user)
        .json(&json!({
            "school": "Test School",
            "class": 9,
        }))
        .send()
        .await;

    assert_error!(res, error::USER_ALREADY_EXISTS);
}
