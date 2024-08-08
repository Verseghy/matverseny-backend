mod utils;

use utils::prelude::*;

mod time {
    use super::*;

    #[tokio::test]
    async fn put_not_admin() {
        let env = setup().await;
        let user = iam::register_user(&env).await;

        let res = env.put("/competition/time").user(&user).send().await;

        assert_error!(res, error::NOT_ENOUGH_PERMISSIONS);
    }

    #[tokio::test]
    async fn patch_not_admin() {
        let env = setup().await;
        let user = iam::register_user(&env).await;

        let res = env.patch("/competition/time").user(&user).send().await;

        assert_error!(res, error::NOT_ENOUGH_PERMISSIONS);
    }

    #[tokio::test]
    async fn success_start_time() {
        let env = setup().await;

        let user = iam::register_user(&env).await;
        iam::make_admin(&env, &user).await;

        let res = env
            .patch("/competition/time")
            .user(&user)
            .json(&json!({
                "start_time": 123,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = env.get("/competition/time").user(&user).send().await;

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
    async fn success_end_time() {
        let env = setup().await;

        let user = iam::register_user(&env).await;
        iam::make_admin(&env, &user).await;

        let res = env
            .patch("/competition/time")
            .user(&user)
            .json(&json!({
                "end_time": 123,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = env.get("/competition/time").user(&user).send().await;

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
    async fn success_both_time() {
        let env = setup().await;

        let user = iam::register_user(&env).await;
        iam::make_admin(&env, &user).await;

        let res = env
            .patch("/competition/time")
            .user(&user)
            .json(&json!({
                "start_time": 432,
                "end_time": 234,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = env.get("/competition/time").user(&user).send().await;

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
    async fn success_socket_events() {
        let env = setup().await;

        let admin = iam::register_user(&env).await;
        iam::make_admin(&env, &admin).await;

        let owner = env.register_user().await;
        let _ = env.create_team(&owner).await;
        let mut socket = env.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let res = env
            .put("/competition/time")
            .user(&admin)
            .json(&json!({
                "start_time": 1234,
                "end_time": 4321,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let message = utils::get_socket_message(socket.next().await);

        assert_json_eq!(
            message,
            json!({
                "event": "UPDATE_TIME",
                "data": {
                    "start_time": 1234,
                    "end_time": 4321,
                }
            })
        );

        socket.close(None).await.unwrap();
    }
}
