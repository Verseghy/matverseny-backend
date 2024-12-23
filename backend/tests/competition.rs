use test_utils::prelude::*;

mod time {
    use super::*;

    #[tokio::test]
    #[parallel]
    async fn put_not_admin() {
        let app = get_cached_app().await;
        let user = iam::register_user().await;

        let res = app.put("/v1/competition/time").user(&user).send().await;

        assert_error!(res, error::NOT_ENOUGH_PERMISSIONS);
    }

    #[tokio::test]
    #[parallel]
    async fn patch_not_admin() {
        let app = get_cached_app().await;
        let user = iam::register_user().await;

        let res = app.patch("/v1/competition/time").user(&user).send().await;

        assert_error!(res, error::NOT_ENOUGH_PERMISSIONS);
    }

    #[tokio::test]
    #[serial]
    async fn success_start_time() {
        let app = get_cached_app().await;

        let user = iam::register_user().await;
        iam::make_admin(&user).await;

        let res = app
            .patch("/v1/competition/time")
            .user(&user)
            .json(&json!({
                "start_time": 123,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app.get("/v1/competition/time").user(&user).send().await;

        assert_eq!(res.status(), StatusCode::OK);

        let body: Value = res.json().await;

        assert_json_include!(
            actual: body,
            expected: json!({
                "start_time": 123,
            })
        );
    }

    #[tokio::test]
    #[serial]
    async fn success_end_time() {
        let app = get_cached_app().await;

        let user = iam::register_user().await;
        iam::make_admin(&user).await;

        let res = app
            .patch("/v1/competition/time")
            .user(&user)
            .json(&json!({
                "end_time": 123,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app.get("/v1/competition/time").user(&user).send().await;

        assert_eq!(res.status(), StatusCode::OK);

        let body: Value = res.json().await;

        assert_json_include!(
            actual: body,
            expected: json!({
                "end_time": 123,
            })
        );
    }

    #[tokio::test]
    #[serial]
    async fn success_both_time() {
        let app = get_cached_app().await;

        let user = iam::register_user().await;
        iam::make_admin(&user).await;

        let res = app
            .patch("/v1/competition/time")
            .user(&user)
            .json(&json!({
                "start_time": 432,
                "end_time": 234,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app.get("/v1/competition/time").user(&user).send().await;

        assert_eq!(res.status(), StatusCode::OK);

        let body: Value = res.json().await;

        assert_json_eq!(
            body,
            json!({
                "start_time": 432,
                "end_time": 234,
            })
        );
    }

    #[tokio::test]
    #[serial]
    async fn success_socket_events() {
        let app = get_cached_app().await;

        let admin = iam::register_user().await;
        iam::make_admin(&admin).await;

        let owner = app.register_user().await;
        let _ = app.create_team(&owner).await;
        let mut socket = app.socket("/v1/ws").start().await;
        assert_team_info!(socket, owner);

        let res = app
            .put("/v1/competition/time")
            .user(&admin)
            .json(&json!({
                "start_time": 1234,
                "end_time": 4321,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let message = get_socket_message(socket.next().await);

        assert_json_eq!(
            message,
            json!({
                "event": "UPDATE_TIME",
                "data": {
                    "start_time": "1970-01-01T00:20:34Z",
                    "end_time": "1970-01-01T01:12:01Z",
                }
            })
        );

        socket.close(None).await.unwrap();
    }
}
