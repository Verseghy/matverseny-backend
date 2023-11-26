mod utils;

use chrono::Utc;
use utils::prelude::*;

#[tokio::test]
#[parallel]
async fn not_admin() {
    let app = get_cached_app().await;
    let user = app.register_user().await;

    let res = app
        .post("/stats")
        .user(&user)
        .json(&json!({
            "timestamp": 12312312,
        }))
        .send()
        .await;

    assert_error!(res, error::NOT_ENOUGH_PERMISSIONS);
}

#[tokio::test]
#[serial]
async fn empty() {
    let app = get_cached_app().await;
    app.clean_database().await;

    let admin_user = utils::iam::register_user().await;
    utils::iam::make_admin(&admin_user).await;

    let time = Utc::now().to_rfc3339();

    let res = app
        .post("/stats")
        .user(&admin_user)
        .json(&json!({
            "timestamp": time,
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let body: Value = res.json().await;

    assert_json_include!(
        actual: body,
        expected: json!([])
    );
}

#[tokio::test]
#[serial]
async fn works() {
    let app = get_cached_app().await;
    app.clean_database().await;

    let admin_user = utils::iam::register_user().await;
    utils::iam::make_admin(&admin_user).await;

    // Setup
    let res = app
        .post("/problem")
        .user(&admin_user)
        .json(&json!({
            "body": "some body",
            "solution": 23,
            "image": "image link",
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let body: Value = res.json().await;
    let id = body.get("id").unwrap();

    let owner = app.register_user().await;
    let team = app.create_team(&owner).await;

    team.lock().await;

    let res = app
        .post("/competition/solution")
        .user(&owner)
        .json(&json!({
            "problem": id,
            "solution": 23,
        }))
        .send()
        .await;

    assert!(res.status().is_success());

    let time = Utc::now().to_rfc3339();

    let res = app
        .post("/stats")
        .user(&admin_user)
        .json(&json!({
            "timestamp": time,
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let body: Value = res.json().await;

    assert_json_include!(
        actual: body,
        expected: json!([{
            "correct": 1,
            "wrong": 0,
        }])
    );
}
