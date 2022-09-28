mod utils;

use utils::prelude::*;

#[tokio::test]
async fn success() {
    let app = App::new().await;
    let user = app.register_user().await;

    let res = app
        .post("/team/create")
        .user(&user)
        .json(&json!({
            "name": "Test Team",
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = res.json::<Value>().await;

    assert!(body.is_object());
    assert!(body["id"].is_string());
}
#[tokio::test]
async fn name_already_taken() {
    let app = App::new().await;
    let user = app.register_user().await;

    let res = app
        .post("/team/create")
        .user(&user)
        .json(&json!({
            "name": "Test Team",
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = res.json::<Value>().await;

    assert!(body.is_object());
    assert!(body["id"].is_string());

    let user2 = app.register_user().await;

    let res = app
        .post("/team/create")
        .user(&user2)
        .json(&json!({
            "name": "Test Team",
        }))
        .send()
        .await;

    assert_error!(res, error::DUPLICATE_TEAM_NAME);
}

#[tokio::test]
async fn already_in_team() {
    let app = App::new().await;
    let user = app.register_user().await;

    let res = app
        .post("/team/create")
        .user(&user)
        .json(&json!({
            "name": "Test Team",
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = res.json::<Value>().await;

    assert!(body.is_object());
    assert!(body["id"].is_string());

    let res = app
        .post("/team/create")
        .user(&user)
        .json(&json!({
            "name": "Test Team",
        }))
        .send()
        .await;

    assert_error!(res, error::ALREADY_IN_TEAM);
}

#[tokio::test]
async fn not_registered() {
    let app = App::new().await;
    let user = utils::iam::register_user().await;

    let res = app
        .post("/team/create")
        .user(&user)
        .json(&json!({
            "name": "Test Team",
        }))
        .send()
        .await;

    assert_error!(res, error::USER_NOT_REGISTERED)
}
