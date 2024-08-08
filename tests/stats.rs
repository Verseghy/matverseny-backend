mod utils;

use chrono::Utc;
use utils::prelude::*;

#[tokio::test]
async fn not_admin() {
    let app = setup().await;
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
async fn empty() {
    let env = setup().await;

    let admin_user = iam::register_user(&env).await;
    iam::make_admin(&env, &admin_user).await;

    let time = Utc::now().to_rfc3339();

    let res = env
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
async fn works() {
    let env = setup().await;

    let admin_user = iam::register_user(&env).await;
    iam::make_admin(&env, &admin_user).await;

    // Setup
    let res = env
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

    let owner = env.register_user().await;
    let team = env.create_team(&owner).await;

    team.lock().await;

    let res = env
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

    let res = env
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

#[tokio::test]
async fn works_after_delete() {
    let env = setup().await;

    let admin_user = iam::register_user(&env).await;
    iam::make_admin(&env, &admin_user).await;

    // Setup
    let res = env
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

    let owner = env.register_user().await;
    let team = env.create_team(&owner).await;

    team.lock().await;

    let res = env
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

    let res = env
        .post("/competition/solution")
        .user(&owner)
        .json(&json!({
            "problem": id,
            "solution": null,
        }))
        .send()
        .await;

    assert!(res.status().is_success());

    let time2 = Utc::now().to_rfc3339();

    let res = env
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

    let res = env
        .post("/stats")
        .user(&admin_user)
        .json(&json!({
            "timestamp": time2,
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let body: Value = res.json().await;

    assert_json_include!(
        actual: body,
        expected: json!([{
            "correct": 0,
            "wrong": 0,
        }])
    );
}

#[tokio::test]
async fn works_with_wrong() {
    let env = setup().await;

    let admin_user = iam::register_user(&env).await;
    iam::make_admin(&env, &admin_user).await;

    // Setup
    let res = env
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

    let owner = env.register_user().await;
    let team = env.create_team(&owner).await;

    team.lock().await;

    let res = env
        .post("/competition/solution")
        .user(&owner)
        .json(&json!({
            "problem": id,
            "solution": 22,
        }))
        .send()
        .await;

    assert!(res.status().is_success());

    let time = Utc::now().to_rfc3339();

    let res = env
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
            "correct": 0,
            "wrong": 1,
        }])
    );
}
