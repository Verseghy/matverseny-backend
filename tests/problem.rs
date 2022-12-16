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
