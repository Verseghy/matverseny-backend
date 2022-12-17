mod utils;

use utils::prelude::*;

mod create {
    use super::*;

    #[tokio::test]
    #[parallel]
    async fn success() {
        let app = get_cached_app().await;
        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let res = app
            .post("/problem")
            .user(&user)
            .json(&json!({
                "body": "some body",
                "solution": 23,
                "image": "image link",
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);

        let body: Value = res.json().await;
        assert!(body.get("id").is_some());
    }

    #[tokio::test]
    #[parallel]
    async fn optional_image() {
        let app = get_cached_app().await;
        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let res = app
            .post("/problem")
            .user(&user)
            .json(&json!({
                "body": "some body",
                "solution": 23,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    #[parallel]
    #[ignore]
    async fn not_admin() {
        let app = get_cached_app().await;
        let user = utils::iam::register_user().await;

        let _res = app
            .post("/problem")
            .user(&user)
            .json(&json!({
                "body": "some body",
                "solution": 32,
            }))
            .send()
            .await;

        //TODO: assert error
    }
}

mod get {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn no_such_problem() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;

        let res = app
            .get(&format!("/problem/{}", uuid()))
            .user(&user)
            .send()
            .await;

        assert_error!(res, error::PROBLEM_NOT_FOUND);
    }

    #[tokio::test]
    #[parallel]
    async fn success() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let res = app
            .post("/problem")
            .user(&user)
            .json(&json!({
                "body": "Test body.",
                "solution": 123,
                "image": "test image",
            }))
            .send()
            .await;

        assert_eq!(
            res.status(),
            StatusCode::CREATED,
            "failed to create problem: response={:#?}",
            res.json::<Value>().await
        );

        let body: Value = res.json().await;
        let id = body["id"].as_str().unwrap();

        let res = app
            .get(&format!("/problem/{}", id))
            .user(&user)
            .send()
            .await;

        assert_eq!(
            res.status(),
            StatusCode::OK,
            "failed to get problem: id={}, response={:#?}",
            id,
            res.json::<Value>().await
        );

        let body: Value = res.json().await;
        assert_json_eq!(
            body,
            json!({
                "id": id,
                "body": "Test body.",
                "solution": 123,
                "image": "test image",
            })
        );
    }

    #[tokio::test]
    #[parallel]
    async fn not_uuid() {
        let app = get_cached_app().await;
        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let res = app.get("/problem/test").user(&user).send().await;

        assert_error!(res, error::PROBLEM_NOT_FOUND);
    }

    #[tokio::test]
    #[parallel]
    #[ignore]
    async fn not_admin() {
        let app = get_cached_app().await;
        let user = utils::iam::register_user().await;

        let _res = app
            .get(&format!("/problem/{}", uuid()))
            .user(&user)
            .send()
            .await;

        //TODO: assert error
    }
}

mod list {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn success() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let res = app.get("/problem").user(&user).send().await;

        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = res.json().await;
        assert_json_eq!(body, json!([]));

        let res = app
            .post("/problem")
            .user(&user)
            .json(&json!({
                "body": "Test body 1.",
                "solution": 1,
                "image": "test image 1",
            }))
            .send()
            .await;

        assert_eq!(
            res.status(),
            StatusCode::CREATED,
            "failed to create problem: response={:#?}",
            res.json::<Value>().await
        );

        let body: Value = res.json().await;
        let id1 = body["id"].as_str().unwrap();

        let res = app.get("/problem").user(&user).send().await;

        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = res.json().await;
        assert_json_eq!(
            body,
            json!([{
                "id": id1,
                "body": "Test body 1.",
                "solution": 1,
                "image": "test image 1",
            }])
        );

        let res = app
            .post("/problem")
            .user(&user)
            .json(&json!({
                "body": "Test body 2.",
                "solution": 2,
                "image": "test image 2",
            }))
            .send()
            .await;

        assert_eq!(
            res.status(),
            StatusCode::CREATED,
            "failed to create problem: response={:#?}",
            res.json::<Value>().await
        );

        let body: Value = res.json().await;
        let id2 = body["id"].as_str().unwrap();

        let res = app.get("/problem").user(&user).send().await;

        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = res.json().await;
        assert_json_eq!(
            body,
            json!([
                {
                    "id": id1,
                    "body": "Test body 1.",
                    "solution": 1,
                    "image": "test image 1",
                },
                {
                    "id": id2,
                    "body": "Test body 2.",
                    "solution": 2,
                    "image": "test image 2",
                },
            ])
        );
    }

    #[tokio::test]
    #[parallel]
    #[ignore]
    async fn not_admin() {
        let app = get_cached_app().await;
        let user = utils::iam::register_user().await;

        let _res = app.get("/problem").user(&user).send().await;

        // TODO: assert error
    }
}
