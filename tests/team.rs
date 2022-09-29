mod utils;

use utils::prelude::*;

mod create {
    use super::*;

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
}

mod join {
    use super::*;

    #[tokio::test]
    async fn success() {
        let app = App::new().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let join_code = team.get_code().await;

        let user = app.register_user().await;

        let res = app
            .post("/team/join")
            .user(&user)
            .json(&json!({
                "code": join_code,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn wrong_code() {
        let app = App::new().await;
        let user = app.register_user().await;

        let res = app
            .post("/team/join")
            .user(&user)
            .json(&json!({
                "code": "AAAAAA",
            }))
            .send()
            .await;

        assert_error!(res, error::JOIN_CODE_NOT_FOUND);
    }

    #[tokio::test]
    async fn already_in_team() {
        let app = App::new().await;
        let user1 = app.register_user().await;
        let _team1 = app.create_team(&user1).await;

        let user2 = app.register_user().await;
        let team2 = app.create_team(&user2).await;

        let res = app
            .post("/team/join")
            .user(&user1)
            .json(&json!({
                "code": team2.get_code().await,
            }))
            .send()
            .await;

        assert_error!(res, error::ALREADY_IN_TEAM);
    }

    //TODO: test locked team
}
