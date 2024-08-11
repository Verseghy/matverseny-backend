mod utils;

use utils::prelude::*;

mod create {
    use super::*;

    #[tokio::test]
    async fn success() {
        let env = setup().await;
        let user = env.register_user().await;

        let res = env
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
    async fn success_with_trailing_slash() {
        let env = setup().await;
        let user = env.register_user().await;

        let res = env
            .post("/team/create/")
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
        let env = setup().await;
        let user = env.register_user().await;

        let res = env
            .post("/team/create")
            .user(&user)
            .json(&json!({
                "name": "Test Team",
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);

        let user2 = env.register_user().await;

        let res = env
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
        let env = setup().await;
        let user = env.register_user().await;

        let res = env
            .post("/team/create")
            .user(&user)
            .json(&json!({
                "name": uuid(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);

        let res = env
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
        let env = setup().await;
        let user = iam::register_user(&env).await;

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let mut socket = env.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let join_code = team.get_code().await;
        let user = env.register_user().await;

        let res = env
            .post("/team/join")
            .user(&user)
            .json(&json!({
                "code": join_code,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::OK);

        let message = utils::get_socket_message(socket.next().await);

        let user_info = libiam::testing::users::get_user(&env.iam_db, &user.id).await;

        assert_json_eq!(
            message,
            json!({
                "event": "JOIN_TEAM",
                "data": {
                    "user": user.id.strip_prefix("UserID-").unwrap(),
                    "name": user_info.name,
                }
            })
        );

        socket.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn wrong_code() {
        let env = setup().await;
        let user = env.register_user().await;

        let res = env
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
        let env = setup().await;
        let user1 = env.register_user().await;
        let _team1 = env.create_team(&user1).await;

        let user2 = env.register_user().await;
        let team2 = env.create_team(&user2).await;

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        team.lock().await;

        let user = env.register_user().await;

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        let mut socket = env.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let res = env.post("/team/leave").user(&member).send().await;
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
        let env = setup().await;
        let owner = env.register_user().await;
        let _team = env.create_team(&owner).await;

        let user = env.register_user().await;

        let res = env.post("/team/leave").user(&user).send().await;

        assert_error!(res, error::USER_NOT_IN_TEAM);
    }

    #[tokio::test]
    async fn locked_team() {
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        team.lock().await;

        let res = env.post("/team/leave").user(&member).send().await;
        assert_error!(res, error::LOCKED_TEAM);
    }

    #[tokio::test]
    async fn owner_cannot_leave() {
        let env = setup().await;
        let owner = env.register_user().await;
        let _team = env.create_team(&owner).await;

        let res = env.post("/team/leave").user(&owner).send().await;

        assert_error!(res, error::OWNER_CANNOT_LEAVE);
    }
}

mod update {
    use super::*;

    #[tokio::test]
    async fn should_not_error_when_empty_json() {
        let env = setup().await;
        let user = env.register_user().await;
        let _team = env.create_team(&user).await;

        let res = env.patch("/team").user(&user).json(&json!({})).send().await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn must_be_owner() {
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        let res = env
            .patch("/team")
            .user(&member)
            .json(&json!({
                "owner": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_error!(res, error::USER_NOT_OWNER);

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let _team = env.create_team(&owner).await;

        let res = env
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": uuid::Uuid::nil(),
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let _team = env.create_team(&owner).await;

        let user = env.register_user().await;

        let res = env
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": user.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_error!(res, error::NO_SUCH_MEMBER);

        let res = env
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
        let env = setup().await;
        let user = env.register_user().await;

        let res = env
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
        let env = setup().await;
        let user = env.register_user().await;
        let _team = env.create_team(&user).await;

        let res = env
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
        let env = setup().await;
        let user = env.register_user().await;
        let _team = env.create_team(&user).await;

        let res = env
            .patch("/team")
            .user(&user)
            .json(&json!({
                "locked": true,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = env
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
        let env = setup().await;
        let user = env.register_user().await;
        let _team = env.create_team(&user).await;

        let res = env
            .patch("/team")
            .user(&user)
            .json(&json!({
                "locked": true,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = env
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
        let env = setup().await;
        let user = env.register_user().await;
        let _team = env.create_team(&user).await;

        let mut socket = env.socket("/ws").start().await;
        assert_team_info!(socket, user);

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        let mut socket = env.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let member_uuid = member.id.strip_prefix("UserID-").unwrap();

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        let mut socket = env.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let member_id = member.id.strip_prefix("UserID-").unwrap();

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let _team = env.create_team(&owner).await;

        let mut socket = env.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let res = env
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
        let env = setup().await;
        let user = env.register_user().await;

        let res = env.post("/team/disband").user(&user).send().await;

        assert_error!(res, error::USER_NOT_IN_TEAM);
    }

    #[tokio::test]
    async fn not_owner() {
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        let res = env.post("/team/disband").user(&member).send().await;

        assert_error!(res, error::USER_NOT_OWNER);
    }

    #[tokio::test]
    async fn locked_team() {
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        team.lock().await;

        let res = env.post("/team/disband").user(&owner).send().await;

        assert_error!(res, error::LOCKED_TEAM);
    }

    #[tokio::test]
    async fn success() {
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member1 = env.register_user().await;
        member1.join(&team.get_code().await).await;

        let member2 = env.register_user().await;
        member2.join(&team.get_code().await).await;

        let mut socket1 = env.socket("/ws").start().await;
        assert_team_info!(socket1, owner);
        let mut socket2 = env.socket("/ws").start().await;
        assert_team_info!(socket2, member1);
        let mut socket3 = env.socket("/ws").start().await;
        assert_team_info!(socket3, member2);

        let res = env.post("/team/disband").user(&owner).send().await;
        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = env.post("/team/leave").user(&owner).send().await;
        assert_error!(res, error::USER_NOT_IN_TEAM);
        let message = socket1.next().await;
        assert_close_frame!(
            message,
            Normal,
            {
                "event": "DISBAND_TEAM",
            },
        );

        let res = env.post("/team/leave").user(&member1).send().await;
        assert_error!(res, error::USER_NOT_IN_TEAM);
        let message = socket2.next().await;
        assert_close_frame!(
            message,
            Normal,
            {
                "event": "DISBAND_TEAM",
            },
        );

        let res = env.post("/team/leave").user(&member2).send().await;
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
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member1 = env.register_user().await;
        member1.join(&team.get_code().await).await;

        let member2 = env.register_user().await;
        member2.join(&team.get_code().await).await;

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        team.lock().await;

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        let res = env
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        let res = env
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let _team = env.create_team(&owner).await;

        let member = env.register_user().await;

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let _team = env.create_team(&owner).await;

        let res = env
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
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        let mut socket1 = env.socket("/ws").start().await;
        assert_team_info!(socket1, owner);
        let mut socket2 = env.socket("/ws").start().await;
        assert_team_info!(socket2, member);

        let res = env
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
                "event": "LEAVE_TEAM",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            })
        );

        assert_close_frame!(
            socket2.next().await,
            Normal,
            {
                "event": "LEAVE_TEAM",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            },
        );

        socket1.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn success_owner_kick_coowner() {
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        let res = env
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": member.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let mut socket1 = env.socket("/ws").start().await;
        assert_team_info!(socket1, owner);
        let mut socket2 = env.socket("/ws").start().await;
        assert_team_info!(socket2, member);

        let res = env
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
                "event": "LEAVE_TEAM",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            })
        );

        assert_close_frame!(
            socket2.next().await,
            Normal,
            {
                "event": "LEAVE_TEAM",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            },
        );

        socket1.close(None).await.unwrap();
    }

    #[tokio::test]
    async fn success_coowner_kick_member() {
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let coowner = env.register_user().await;
        coowner.join(&team.get_code().await).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        let res = env
            .patch("/team")
            .user(&owner)
            .json(&json!({
                "co_owner": coowner.id.strip_prefix("UserID-").unwrap(),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let mut socket1 = env.socket("/ws").start().await;
        assert_team_info!(socket1, owner);
        let mut socket2 = env.socket("/ws").start().await;
        assert_team_info!(socket2, coowner);
        let mut socket3 = env.socket("/ws").start().await;
        assert_team_info!(socket3, member);

        let res = env
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
                "event": "LEAVE_TEAM",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            })
        );

        assert_json_eq!(
            utils::get_socket_message(socket2.next().await),
            json!({
                "event": "LEAVE_TEAM",
                "data": {
                    "user": member.id.strip_prefix("UserID-").unwrap(),
                }
            })
        );

        assert_close_frame!(
            socket3.next().await,
            Normal,
            {
                "event": "LEAVE_TEAM",
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
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        let member = env.register_user().await;
        member.join(&team.get_code().await).await;

        let res = env.post("/team/code").user(&member).send().await;

        assert_error!(res, error::USER_NOT_COOWNER);
    }

    #[tokio::test]
    async fn locked_team() {
        let env = setup().await;
        let owner = env.register_user().await;
        let team = env.create_team(&owner).await;

        team.lock().await;

        let res = env.post("/team/code").user(&owner).send().await;

        assert_error!(res, error::LOCKED_TEAM);
    }

    #[tokio::test]
    async fn success() {
        let env = setup().await;
        let owner = env.register_user().await;
        let _team = env.create_team(&owner).await;

        let mut socket = env.socket("/ws").start().await;
        assert_team_info!(socket, owner);

        let res = env.post("/team/code").user(&owner).send().await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        let message = utils::get_socket_message(socket.next().await);

        assert_eq!(message["event"].as_str().unwrap(), "UPDATE_TEAM");
        assert!(!message["data"]["code"].as_str().unwrap().is_empty());

        socket.close(None).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn clash() {
        // TODO: test join code clash
    }
}

mod get {
    use super::*;

    #[tokio::test]
    async fn not_admin() {
        let env = setup().await;
        let user = env.register_user().await;

        let res = env.get("/team").user(&user).send().await;

        assert_error!(res, error::NOT_ENOUGH_PERMISSIONS);
    }

    #[tokio::test]
    async fn works() {
        let env = setup().await;

        let owner = env.register_user().await;
        let _team = env.create_team(&owner).await;

        let admin = iam::register_user(&env).await;
        iam::make_admin(&env, &admin).await;

        let res = env.get("/team").user(&admin).send().await;

        assert!(res.status().is_success());

        let body: Value = res.json().await;

        assert!(body.is_array());

        let first = &body.as_array().unwrap()[0];

        assert!(first.get("id").is_some());
        assert!(first.get("name").is_some());
        assert!(first.get("join_code").is_some());

        let owner_id = owner.id.strip_prefix("UserID-").unwrap();

        assert_json_include!(
            actual: body,
            expected: json!([
                {
                    "owner": owner_id,
                    "co_owner": null,
                    "locked": false,
                    "members": [
                        {
                            "id": owner_id,
                            "school": "Test School",
                            "class": 9,
                        }
                    ],
                }
            ])
        )
    }
}
