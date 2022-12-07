mod utils;

use utils::prelude::*;

mod create {
    use super::*;

    #[tokio::test]
    async fn success() {
        let app = get_cached_app().await;
        let user = app.register_user().await;

        let res = app
            .post("/team/create")
            .user(&user)
            .json(&json!({
                "name": uuid(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn name_already_taken() {
        let app = get_cached_app().await;
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
        let app = get_cached_app().await;
        let user = app.register_user().await;

        let res = app
            .post("/team/create")
            .user(&user)
            .json(&json!({
                "name": uuid(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);

        let res = app
            .post("/team/create")
            .user(&user)
            .json(&json!({
                "name": uuid(),
            }))
            .send()
            .await;

        assert_error!(res, error::ALREADY_IN_TEAM);
    }

    #[tokio::test]
    async fn not_registered() {
        let app = get_cached_app().await;
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
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let mut socket = app.socket("/ws").start().await;
        assert_team_info!(socket, owner);

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

        assert_json_eq!(
            message,
            json!({
                "event": "JOIN_TEAM",
                "data": {
                    "user": user.id.strip_prefix("UserID-").unwrap(),
                }
            })
        );

        socket.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn wrong_code() {
        let app = get_cached_app().await;
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
        let app = get_cached_app().await;
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
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        team.lock().await;

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
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let mut socket = app.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let res = app.post("/team/leave").user(&member).send().await;
        assert_eq!(res.status(), StatusCode::OK);

        let message = utils::get_socket_message(socket.next().await);

        assert_json_eq!(
            message,
            json!({
                "event": "LEAVE_TEAM",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            })
        );

        socket.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn not_in_team() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let user = app.register_user().await;

        let res = app.post("/team/leave").user(&user).send().await;

        assert_error!(res, error::USER_NOT_IN_TEAM);
    }

    #[tokio::test]
    async fn locked_team() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        team.lock().await;

        let res = app.post("/team/leave").user(&member).send().await;
        assert_error!(res, error::LOCKED_TEAM);
    }

    #[tokio::test]
    async fn owner_cannot_leave() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let res = app.post("/team/leave").user(&owner).send().await;

        assert_error!(res, error::OWNER_CANNOT_LEAVE);
    }
}

mod update {
    use super::*;

    #[tokio::test]
    async fn should_not_error_when_empty_json() {
        let app = get_cached_app().await;
        let user = app.register_user().await;
        let _team = app.create_team(&user).await;

        let res = app.patch("/team").user(&user).json(&json!({})).send().await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn must_be_owner() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let res = app
            .patch("/team")
            .user(&member)
            .json(&json!({
                "owner": member.id.strip_prefix("UserID-").unwrap(),
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
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": uuid::Uuid::nil(),
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "owner": uuid::Uuid::nil(),
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);
    }

    #[tokio::test]
    async fn existing_user_but_not_a_team_member() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let user = app.register_user().await;

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": user.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "owner": user.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);
    }

    #[tokio::test]
    async fn not_in_team() {
        let app = get_cached_app().await;
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
        let app = get_cached_app().await;
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
        let app = get_cached_app().await;
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
        let app = get_cached_app().await;
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
        let app = get_cached_app().await;
        let user = app.register_user().await;
        let _team = app.create_team(&user).await;

        let mut socket = app.socket("/ws").start().await;
        assert_team_info!(socket, user);

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

        assert_json_eq!(
            message,
            json!({
                "event": "UPDATE_TEAM",
                "data": {
                    "name": "new name",
                }
            })
        );

        socket.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn success_owner() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let mut socket = app.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let member_uuid = member.id.strip_prefix("UserID-").unwrap();

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "owner": member_uuid,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let message = utils::get_socket_message(socket.next().await);

        assert_json_eq!(
            message,
            json!({
                "event": "UPDATE_TEAM",
                "data": {
                    "owner": member_uuid,
                }
            })
        );

        socket.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn success_coowner() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let mut socket = app.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let member_id = member.id.strip_prefix("UserID-").unwrap();

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": member_id,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let message = socket.next().await;
        let message = utils::get_socket_message(message);

        assert_json_eq!(
            message,
            json!({
                "event": "UPDATE_TEAM",
                "data": {
                    "co_owner": member_id,
                }
            })
        );

        socket.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn delete_coowner() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let mut socket = app.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({ "co_owner": null }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let message = utils::get_socket_message(socket.next().await);

        assert_json_eq!(
            message,
            json!({
                "event": "UPDATE_TEAM",
                "data": {
                    "co_owner": null,
                }
            })
        );

        socket.close(None).await.unwrap();
    }
}

mod disband {
    use super::*;

    #[tokio::test]
    async fn not_in_team() {
        let app = get_cached_app().await;
        let user = app.register_user().await;

        let res = app.post("/team/disband").user(&user).send().await;

        assert_error!(res, error::USER_NOT_IN_TEAM);
    }

    #[tokio::test]
    async fn not_owner() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let res = app.post("/team/disband").user(&member).send().await;

        assert_error!(res, error::USER_NOT_OWNER);
    }

    #[tokio::test]
    async fn locked_team() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        team.lock().await;

        let res = app.post("/team/disband").user(&owner).send().await;

        assert_error!(res, error::LOCKED_TEAM);
    }

    #[tokio::test]
    async fn success() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member1 = app.register_user().await;
        member1.join(&team.get_code().await).await;

        let member2 = app.register_user().await;
        member2.join(&team.get_code().await).await;

        let mut socket1 = app.socket("/ws").start().await;
        assert_team_info!(socket1, owner);
        let mut socket2 = app.socket("/ws").start().await;
        assert_team_info!(socket2, member1);
        let mut socket3 = app.socket("/ws").start().await;
        assert_team_info!(socket3, member2);

        let res = app.post("/team/disband").user(&owner).send().await;
        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app.post("/team/leave").user(&owner).send().await;
        assert_error!(res, error::USER_NOT_IN_TEAM);
        let message = socket1.next().await;
        assert_close_frame!(
            message,
            Normal,
            {
                "event": "DISBAND_TEAM",
            },
        );

        let res = app.post("/team/leave").user(&member1).send().await;
        assert_error!(res, error::USER_NOT_IN_TEAM);
        let message = socket2.next().await;
        assert_close_frame!(
            message,
            Normal,
            {
                "event": "DISBAND_TEAM",
            },
        );

        let res = app.post("/team/leave").user(&member2).send().await;
        assert_error!(res, error::USER_NOT_IN_TEAM);
        let message = socket3.next().await;
        assert_close_frame!(
            message,
            Normal,
            {
                "event": "DISBAND_TEAM",
            },
        );
    }
}

mod kick {
    use super::*;

    #[tokio::test]
    async fn member_cannot_kick() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member1 = app.register_user().await;
        member1.join(&team.get_code().await).await;

        let member2 = app.register_user().await;
        member2.join(&team.get_code().await).await;

        let res = app
            .post("/team/kick")
            .user(&member1)
            .json(&json!({
                "user": member2.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_error!(res, error::USER_NOT_COOWNER)
    }

    #[tokio::test]
    async fn locked_team() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        team.lock().await;

        let res = app
            .post("/team/kick")
            .user(&owner)
            .json(&json!({
                "user": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_error!(res, error::LOCKED_TEAM);
    }

    #[tokio::test]
    async fn cannot_kick_owner() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app
            .post("/team/kick")
            .user(&member)
            .json(&json!({
                "user": owner.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_error!(res, error::CANNOT_KICK_OWNER);
    }

    #[tokio::test]
    async fn cannot_kick_themself() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = app
            .post("/team/kick")
            .user(&member)
            .json(&json!({
                "user": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_error!(res, error::CANNOT_KICK_THEMSELF);
    }

    #[tokio::test]
    async fn not_member() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let member = app.register_user().await;

        let res = app
            .post("/team/kick")
            .user(&owner)
            .json(&json!({
                "user": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);
    }

    #[tokio::test]
    async fn user_not_exists() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let res = app
            .post("/team/kick")
            .user(&owner)
            .json(&json!({
                "user": uuid::Uuid::nil(),
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);
    }

    #[tokio::test]
    async fn success_owner_kick_member() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let mut socket1 = app.socket("/ws").start().await;
        assert_team_info!(socket1, owner);
        let mut socket2 = app.socket("/ws").start().await;
        assert_team_info!(socket2, member);

        let res = app
            .post("/team/kick")
            .user(&owner)
            .json(&json!({
                "user": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        assert_json_eq!(
            utils::get_socket_message(socket1.next().await),
            json!({
                "event": "KICK_USER",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            })
        );

        assert_close_frame!(
            socket2.next().await,
            Normal,
            {
                "event": "KICK_USER",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            },
        );

        socket1.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn success_owner_kick_coowner() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let mut socket1 = app.socket("/ws").start().await;
        assert_team_info!(socket1, owner);
        let mut socket2 = app.socket("/ws").start().await;
        assert_team_info!(socket2, member);

        let res = app
            .post("/team/kick")
            .user(&owner)
            .json(&json!({
                "user": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        assert_json_eq!(
            utils::get_socket_message(socket1.next().await),
            json!({
                "event": "KICK_USER",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            })
        );

        assert_close_frame!(
            socket2.next().await,
            Normal,
            {
                "event": "KICK_USER",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            },
        );

        socket1.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn success_coowner_kick_member() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let coowner = app.register_user().await;
        coowner.join(&team.get_code().await).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let res = app
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": coowner.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let mut socket1 = app.socket("/ws").start().await;
        assert_team_info!(socket1, owner);
        let mut socket2 = app.socket("/ws").start().await;
        assert_team_info!(socket2, coowner);
        let mut socket3 = app.socket("/ws").start().await;
        assert_team_info!(socket3, member);

        let res = app
            .post("/team/kick")
            .user(&coowner)
            .json(&json!({
                "user": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        assert_json_eq!(
            utils::get_socket_message(socket1.next().await),
            json!({
                "event": "KICK_USER",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            })
        );

        assert_json_eq!(
            utils::get_socket_message(socket2.next().await),
            json!({
                "event": "KICK_USER",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            })
        );

        assert_close_frame!(
            socket3.next().await,
            Normal,
            {
                "event": "KICK_USER",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            },
        );

        socket1.close(None).await.unwrap();
        socket2.close(None).await.unwrap();
    }
}

mod code {
    use super::*;

    #[tokio::test]
    async fn not_coowner() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        let member = app.register_user().await;
        member.join(&team.get_code().await).await;

        let res = app.post("/team/code").user(&member).send().await;

        assert_error!(res, error::USER_NOT_COOWNER);
    }

    #[tokio::test]
    async fn locked_team() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let team = app.create_team(&owner).await;

        team.lock().await;

        let res = app.post("/team/code").user(&owner).send().await;

        assert_error!(res, error::LOCKED_TEAM);
    }

    #[tokio::test]
    async fn success() {
        let app = get_cached_app().await;
        let owner = app.register_user().await;
        let _team = app.create_team(&owner).await;

        let mut socket = app.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let res = app.post("/team/code").user(&owner).send().await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let message = utils::get_socket_message(socket.next().await);

        assert_eq!(message["event"].as_str().unwrap(), "UPDATE_TEAM");
        assert!(!message["data"]["code"].as_str().unwrap().is_empty());

        socket.close(None).await.unwrap();
    }

    // TODO: test join code clash
}
