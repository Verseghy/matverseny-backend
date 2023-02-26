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
    #[serial]
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

mod delete {
    use super::*;

    #[tokio::test]
    #[parallel]
    async fn not_found() {
        let app = get_cached_app().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let res = app
            .delete("/problem")
            .user(&user)
            .json(&json!({
                "id": uuid(),
            }))
            .send()
            .await;

        assert_error!(res, error::PROBLEM_NOT_FOUND);
    }

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
            res.json::<Value>().await,
        );

        let body: Value = res.json().await;
        let id = body["id"].as_str().unwrap();

        let res = app
            .delete("/problem")
            .user(&user)
            .json(&json!({
                "id": id,
            }))
            .send()
            .await;

        assert_eq!(
            res.status(),
            StatusCode::OK,
            "failed to delete problem: response={:#?}",
            res.json::<Value>().await,
        );
    }

    #[tokio::test]
    #[parallel]
    #[ignore]
    async fn not_admin() {
        let app = get_cached_app().await;
        let user = utils::iam::register_user().await;

        let _res = app.delete("/problem").user(&user).send().await;

        // TODO: assert error
    }
}

mod update {
    use super::*;

    #[tokio::test]
    #[parallel]
    async fn not_found() {
        let app = get_cached_app().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let res = app
            .patch("/problem")
            .user(&user)
            .json(&json!({
                "id": uuid(),
                "body": "Test body 1.",
                "solution": 1,
                "image": "test image 1",
            }))
            .send()
            .await;

        assert_error!(res, error::PROBLEM_NOT_FOUND);
    }

    #[tokio::test]
    #[parallel]
    async fn delete_image() {
        let app = get_cached_app().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

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
            res.json::<Value>().await,
        );

        let body: Value = res.json().await;
        let id = body["id"].as_str().unwrap();

        let res = app
            .patch("/problem")
            .user(&user)
            .json(&json!({
                "id": id,
                "body": "Test body 2.",
                "solution": 2,
                "image": null,
            }))
            .send()
            .await;

        assert_eq!(
            res.status(),
            StatusCode::NO_CONTENT,
            "failed to update problem: response={:#?}",
            res.json::<Value>().await
        );

        let res = app
            .get(&format!("/problem/{}", id))
            .user(&user)
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::OK);

        assert_json_eq!(
            res.json::<Value>().await,
            json!({
                "id": id,
                "body": "Test body 2.",
                "solution": 2,
            })
        )
    }

    #[tokio::test]
    #[parallel]
    async fn everything_is_optional() {
        let app = get_cached_app().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

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
            res.json::<Value>().await,
        );

        let body: Value = res.json().await;
        let id = body["id"].as_str().unwrap();

        let res = app
            .patch("/problem")
            .user(&user)
            .json(&json!({
                "id": id,
            }))
            .send()
            .await;

        assert_eq!(
            res.status(),
            StatusCode::NO_CONTENT,
            "failed to update problem: response={:#?}",
            res.json::<Value>().await
        );
    }

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
            res.json::<Value>().await,
        );

        let body: Value = res.json().await;
        let id = body["id"].as_str().unwrap();

        let res = app
            .patch("/problem")
            .user(&user)
            .json(&json!({
                "id": id,
                "body": "Test body 2.",
                "solution": 2,
                "image": "test image 2",
            }))
            .send()
            .await;

        assert_eq!(
            res.status(),
            StatusCode::NO_CONTENT,
            "failed to update problem: response={:#?}",
            res.json::<Value>().await
        );

        let res = app
            .get(&format!("/problem/{}", id))
            .user(&user)
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::OK);

        assert_json_eq!(
            res.json::<Value>().await,
            json!({
                "id": id,
                "body": "Test body 2.",
                "solution": 2,
                "image": "test image 2",
            })
        )
    }

    #[tokio::test]
    #[parallel]
    #[ignore]
    async fn not_admin() {
        let app = get_cached_app().await;
        let user = utils::iam::register_user().await;

        let _res = app.delete("/problem").user(&user).send().await;

        // TODO: assert error
    }
}

mod order {
    use uuid::Uuid;

    use super::*;

    async fn create_test_problem2<const N: usize>(app: &App, user: &impl UserLike) -> [Uuid; N] {
        let mut ids = [Uuid::nil(); N];

        for i in 0..N {
            let res = app
                .post("/problem")
                .user(user)
                .json(&json!({
                    "body": "",
                    "solution": 0,
                }))
                .send()
                .await;

            assert_eq!(res.status(), StatusCode::CREATED);

            let id = Uuid::parse_str(res.json::<Value>().await["id"].as_str().unwrap()).unwrap();

            let res = app
                .post("/problem/order")
                .user(user)
                .json(&json!({
                    "type": "INSERT",
                    "id": id,
                }))
                .send()
                .await;

            assert_eq!(res.status(), StatusCode::NO_CONTENT);

            ids[i] = id;
        }

        let order = get_order_list(app, user).await;
        assert_eq!(
            order,
            ids.iter().map(|id| id.to_string()).collect::<Vec<String>>()
        );

        ids
    }

    async fn create_test_problem(app: &App, user: &impl UserLike) -> Uuid {
        let res = app
            .post("/problem")
            .user(user)
            .json(&json!({
                "body": "",
                "solution": 0,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);

        Uuid::parse_str(res.json::<Value>().await["id"].as_str().unwrap()).unwrap()
    }

    #[tokio::test]
    #[parallel]
    async fn insert_not_found() {
        let app = get_cached_app().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "id": uuid(),
            }))
            .send()
            .await;

        assert_error!(res, error::PROBLEM_NOT_FOUND);
    }

    #[tokio::test]
    #[parallel]
    async fn insert_before_not_found_before() {
        let app = get_cached_app().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "before": uuid(),
                "id": uuid(),
            }))
            .send()
            .await;

        assert_error!(res, error::PROBLEM_NOT_FOUND);
    }

    #[tokio::test]
    #[parallel]
    async fn insert_before_not_found_id() {
        let app = get_cached_app().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let id = create_test_problem(&app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "before": id,
                "id": uuid(),
            }))
            .send()
            .await;

        assert_error!(res, error::PROBLEM_NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn insert_before_problem_already_in() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let id = create_test_problem(&app, &user).await;
        let id2 = create_test_problem(&app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "id": id,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "id": id2,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "before": id2,
                "id": id,
            }))
            .send()
            .await;

        assert_error!(res, error::PROBLEM_ALREADY_IN_ORDER);
    }

    #[tokio::test]
    #[serial]
    async fn problem_already_in() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let id = create_test_problem(&app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "id": id,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "id": id,
            }))
            .send()
            .await;

        assert_error!(res, error::PROBLEM_ALREADY_IN_ORDER);
    }

    #[tokio::test]
    #[parallel]
    async fn delete_not_found() {
        let app = get_cached_app().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "DELETE",
                "id": uuid(),
            }))
            .send()
            .await;

        assert_error!(res, error::PROBLEM_NOT_FOUND);
    }

    #[tokio::test]
    #[parallel]
    async fn swap_not_found() {
        let app = get_cached_app().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "SWAP",
                "id1": uuid(),
                "id2": uuid(),
            }))
            .send()
            .await;

        assert_error!(res, error::PROBLEM_NOT_FOUND);
    }

    async fn get_order_list(app: &App, user: &impl UserLike) -> Vec<String> {
        let res = app.get("/problem/order").user(user).send().await;

        assert_eq!(res.status(), StatusCode::OK);

        let json: Value = res.json().await;
        assert!(json.is_array());
        json.as_array()
            .unwrap()
            .iter()
            .map(|v| {
                assert!(v.is_string());
                v.as_str().unwrap().to_owned()
            })
            .collect()
    }

    #[tokio::test]
    #[serial]
    async fn success_insert() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let order = get_order_list(app, &user).await;
        assert!(order.is_empty());

        let id = create_test_problem(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "id": id,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id.to_string()]);

        let id2 = create_test_problem(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "id": id2,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id.to_string(), id2.to_string()]);
    }

    #[tokio::test]
    #[serial]
    async fn success_insert_before_front() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let order = get_order_list(app, &user).await;
        assert!(order.is_empty());

        let id = create_test_problem(app, &user).await;
        let id2 = create_test_problem(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "id": id,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id.to_string()]);

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "before": id,
                "id": id2,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id2.to_string(), id.to_string()]);
    }

    #[tokio::test]
    #[serial]
    async fn success_insert_before_middle() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let order = get_order_list(app, &user).await;
        assert!(order.is_empty());

        let id = create_test_problem(app, &user).await;
        let id2 = create_test_problem(app, &user).await;
        let id3 = create_test_problem(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "id": id,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "id": id3,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id.to_string(), id3.to_string()]);

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "INSERT",
                "before": id3,
                "id": id2,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id.to_string(), id2.to_string(), id3.to_string()]);
    }

    #[tokio::test]
    #[serial]
    async fn success_delete_from_front() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let [id, id2] = create_test_problem2(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "DELETE",
                "id": id,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id2.to_string()]);
    }

    #[tokio::test]
    #[serial]
    async fn success_delete_from_middle() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let [id, id2, id3] = create_test_problem2(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "DELETE",
                "id": id2,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id.to_string(), id3.to_string()]);
    }

    #[tokio::test]
    #[serial]
    async fn success_delete_from_end() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let [id, id2] = create_test_problem2(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "DELETE",
                "id": id2,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id.to_string()]);
    }

    #[tokio::test]
    #[serial]
    async fn success_swap_adjacent_start() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let [id1, id2, id3] = create_test_problem2(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "SWAP",
                "id1": id1,
                "id2": id2,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id2.to_string(), id1.to_string(), id3.to_string()]);
    }

    #[tokio::test]
    #[serial]
    async fn success_swap_adjacent_start2() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let [id1, id2, id3] = create_test_problem2(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "SWAP",
                "id1": id2,
                "id2": id1,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id2.to_string(), id1.to_string(), id3.to_string()]);
    }

    #[tokio::test]
    #[serial]
    async fn success_swap_adjacent() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let [id1, id2, id3] = create_test_problem2(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "SWAP",
                "id1": id2,
                "id2": id3,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id1.to_string(), id3.to_string(), id2.to_string()]);
    }

    #[tokio::test]
    #[serial]
    async fn success_swap_adjacent2() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let [id1, id2, id3] = create_test_problem2(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "SWAP",
                "id1": id3,
                "id2": id2,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id1.to_string(), id3.to_string(), id2.to_string()]);
    }

    #[tokio::test]
    #[serial]
    async fn success_swap() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let [id1, id2, id3, id4] = create_test_problem2(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "SWAP",
                "id1": id2,
                "id2": id4,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(
            order,
            [
                id1.to_string(),
                id4.to_string(),
                id3.to_string(),
                id2.to_string()
            ]
        );
    }

    #[tokio::test]
    #[serial]
    async fn success_swap2() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let [id1, id2, id3, id4] = create_test_problem2(app, &user).await;

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "SWAP",
                "id1": id4,
                "id2": id2,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(
            order,
            [
                id1.to_string(),
                id4.to_string(),
                id3.to_string(),
                id2.to_string()
            ]
        );
    }

    #[tokio::test]
    #[serial]
    async fn success_swap_start() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let [id1, id2, id3] = create_test_problem2(app, &user).await;

        let order = get_order_list(app, &user).await;
        tracing::debug!("original order: {order:?}");

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "SWAP",
                "id1": id1,
                "id2": id3,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id3.to_string(), id2.to_string(), id1.to_string()]);
    }

    #[tokio::test]
    #[serial]
    async fn success_swap_start2() {
        let app = get_cached_app().await;
        app.clean_database().await;

        let user = utils::iam::register_user().await;
        utils::iam::make_admin(&user).await;

        let [id1, id2, id3] = create_test_problem2(app, &user).await;

        let order = get_order_list(app, &user).await;
        tracing::debug!("original order: {order:?}");

        let res = app
            .post("/problem/order")
            .user(&user)
            .json(&json!({
                "type": "SWAP",
                "id1": id3,
                "id2": id1,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let order = get_order_list(app, &user).await;
        assert_eq!(order, [id3.to_string(), id2.to_string(), id1.to_string()]);
    }

    #[tokio::test]
    #[parallel]
    #[ignore]
    async fn not_admin() {
        let app = get_cached_app().await;
        let user = utils::iam::register_user().await;

        let _res = app.delete("/problem").user(&user).send().await;

        // TODO: assert error
    }
}
