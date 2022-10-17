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

        let mut socket = app.socket("/ws").user(&owner).start().await;
        assert_team_info!(socket);

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

        let message = utils::get_socket_message(socket.next().await);

        assert_eq!(
            message,
            json!({
                "event": "JOIN_TEAM",
                "data": {
                    "user": user.id,
                }
            })
        );
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

    #[tokio::test]
    async fn locked_team() {
        let app = App::new().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "locked": true,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let user = app.register_user().await;

        let res = app
            .post("/team/join")
            .user(&user)
            .json(&json!({
                "code": team.get_code().await,
            }))
            .send()
            .await;

        assert_error!(res, error::LOCKED_TEAM);
    }
}

mod leave {
    use super::*;

    #[tokio::test]
    async fn success() {
        let app = App::new().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let mut socket = app.socket("/ws").user(&owner).start().await;
        assert_team_info!(socket);

        let res = app.post("/team/leave").user(&member).send().await;
        assert_eq!(res.status(), StatusCode::OK);

        let message = utils::get_socket_message(socket.next().await);

        assert_eq!(
            message,
            json!({
                "event": "LEAVE_TEAM",
                "data": {
                    "user": member.id,
                }
            })
        );
    }

    #[tokio::test]
    async fn not_in_team() {
        let app = App::new().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let user = app.register_user().await;

        let res = app.post("/team/leave").user(&user).send().await;

        assert_error!(res, error::USER_NOT_IN_TEAM);
    }

    #[tokio::test]
    async fn locked_team() {
        let app = App::new().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "locked": true,
            }))
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app.post("/team/leave").user(&member).send().await;
        assert_error!(res, error::LOCKED_TEAM);
    }
}

mod update {
    use super::*;

    #[tokio::test]
    async fn should_not_error_when_empty_json() {
        let app = App::new().await;
        let user = app.register_user().await;
        let _team = app.create_team(&user).await;

        let res = app.patch("/team").user(&user).json(&json!({})).send().await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn must_be_owner() {
        let app = App::new().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let res = app
            .patch("/team")
            .user(&member)
            .json(&json!({
                "owner": member.id,
            }))
            .send()
            .await;

        assert_error!(res, error::USER_NOT_OWNER);

        let res = app
            .patch("/team")
            .user(&member)
            .json(&json!({
                "coowner": member.id,
            }))
            .send()
            .await;

        assert_error!(res, error::USER_NOT_OWNER);
    }

    #[tokio::test]
    async fn non_existing_user() {
        let app = App::new().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "coowner": format!("UserID-{}", uuid::Uuid::nil()),
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "owner": format!("UserID-{}", uuid::Uuid::nil()),
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);
    }

    #[tokio::test]
    async fn existing_user_but_not_a_team_member() {
        let app = App::new().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let user = app.register_user().await;

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "coowner": user.id,
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "owner": user.id,
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);
    }

    #[tokio::test]
    async fn not_in_team() {
        let app = App::new().await;
        let user = app.register_user().await;

        let res = app
            .patch("/team")
            .user(&user)
            .json(&json!({
                "name": "some cool team name",
            }))
            .send()
            .await;

        assert_error!(res, error::USER_NOT_IN_TEAM);
    }

    #[tokio::test]
    async fn update_while_locking() {
        let app = App::new().await;
        let user = app.register_user().await;
        let _team = app.create_team(&user).await;

        let res = app
            .patch("/team")
            .user(&user)
            .json(&json!({
                "name": "best team name ever",
                "locked": true,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn locked_team() {
        let app = App::new().await;
        let user = app.register_user().await;
        let _team = app.create_team(&user).await;

        let res = app
            .patch("/team")
            .user(&user)
            .json(&json!({
                "locked": true,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app
            .patch("/team")
            .user(&user)
            .json(&json!({
                "name": "the worst team name ever",
            }))
            .send()
            .await;

        assert_error!(res, error::LOCKED_TEAM);
    }

    #[tokio::test]
    async fn update_while_unlocking() {
        let app = App::new().await;
        let user = app.register_user().await;
        let _team = app.create_team(&user).await;

        let res = app
            .patch("/team")
            .user(&user)
            .json(&json!({
                "locked": true,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app
            .patch("/team")
            .user(&user)
            .json(&json!({
                "name": "the worst team name ever",
                "locked": false,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn success_name() {
        let app = App::new().await;
        let user = app.register_user().await;
        let _team = app.create_team(&user).await;

        let mut socket = app.socket("/ws").user(&user).start().await;
        assert_team_info!(socket);

        let res = app
            .patch("/team")
            .user(&user)
            .json(&json!({
                "name": "new name",
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let message = utils::get_socket_message(socket.next().await);

        assert_eq!(
            message,
            json!({
                "event": "UPDATE_TEAM",
                "data": {
                    "name": "new name",
                }
            })
        );
    }

    #[tokio::test]
    async fn success_owner() {
        let app = App::new().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let mut socket = app.socket("/ws").user(&owner).start().await;
        assert_team_info!(socket);

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "owner": member.id,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let message = utils::get_socket_message(socket.next().await);

        assert_eq!(
            message,
            json!({
                "event": "UPDATE_TEAM",
                "data": {
                    "owner": member.id,
                }
            })
        );
    }

    #[tokio::test]
    async fn success_coowner() {
        let app = App::new().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let mut socket = app.socket("/ws").user(&owner).start().await;
        assert_team_info!(socket);

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "coowner": member.id,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let message = utils::get_socket_message(socket.next().await);

        assert_eq!(
            message,
            json!({
                "event": "UPDATE_TEAM",
                "data": {
                    "coowner": member.id,
                }
            })
        );
    }

    #[tokio::test]
    async fn delete_coowner() {
        let app = App::new().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let mut socket = app.socket("/ws").user(&owner).start().await;
        assert_team_info!(socket);

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({ "coowner": null }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let message = utils::get_socket_message(socket.next().await);

        assert_eq!(
            message,
            json!({
                "event": "UPDATE_TEAM",
                "data": {
                    "coowner": null,
                }
            })
        );
    }
}
