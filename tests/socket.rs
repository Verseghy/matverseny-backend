mod utils;

use std::time::Duration;

use chrono::Utc;
use tokio_tungstenite::tungstenite::Message;
use utils::prelude::*;
use uuid::Uuid;

#[tokio::test]
async fn timeout() {
    let env = setup().await;

    let mut socket = env.socket("/ws").start().await;

    assert_close_frame_error!(socket.next().await, error::WEBSOCKET_AUTH_TIMEOUT);
}

#[tokio::test]
async fn wrong_token() {
    let env = setup().await;

    let mut socket = env.socket("/ws").start().await;

    socket
        .send(Message::Text("some random invalid token".to_owned()))
        .await
        .unwrap();
    assert_close_frame_error!(socket.next().await, error::JWT_INVALID_TOKEN);
}

#[tokio::test]
async fn wrong_message_type() {
    let env = setup().await;

    let mut socket = env.socket("/ws").start().await;

    socket
        .send(Message::Binary(Vec::from("asd".as_bytes())))
        .await
        .unwrap();
    assert_close_frame_error!(socket.next().await, error::WEBSOCKET_WRONG_MESSAGE_TYPE);
}

#[tokio::test]
async fn user_not_registered() {
    let env = setup().await;
    let user = iam::register_user(&env).await;

    let mut socket = env.socket("/ws").start().await;

    socket
        .send(Message::Text(
            json!({"token": user.access_token().to_owned()}).to_string(),
        ))
        .await
        .unwrap();
    assert_close_frame_error!(socket.next().await, error::USER_NOT_REGISTERED);
}

#[tokio::test]
async fn no_team() {
    let env = setup().await;
    let user = env.register_user().await;

    let mut socket = env.socket("/ws").start().await;

    socket
        .send(Message::Text(
            json!({"token": user.access_token().to_owned()}).to_string(),
        ))
        .await
        .unwrap();
    assert_close_frame_error!(socket.next().await, error::USER_NOT_IN_TEAM);
}

#[tokio::test]
async fn team_info() {
    let env = setup().await;
    let user = env.register_user().await;

    let team = env.create_team(&user).await;

    let mut socket = env.socket("/ws").start().await;
    socket
        .send(Message::Text(
            json!({"token": user.access_token().to_owned()}).to_string(),
        ))
        .await
        .unwrap();

    let message = utils::get_socket_message(socket.next().await);

    assert_json_include!(
        actual: message,
        expected: json!({
            "event": "TEAM_INFO",
            "data": {
                "name": team.get_name(),
                "members": [{
                    "class": 9,
                    "id": user.id.strip_prefix("UserID-").unwrap(),
                    "rank": "Owner",
                }],
                "locked": false,
            },
        })
    );

    assert!(message["data"].get("code").is_some());
    let user = libiam::testing::users::get_user(env.iam_db(), &user.id).await;
    assert_eq!(message["data"]["members"][0]["name"], user.name);

    socket.close(None).await.unwrap();
}

#[tokio::test]
async fn dont_send_problems_before_start() {
    let env = setup().await;

    let admin = iam::register_user(&env).await;
    iam::make_admin(&env, &admin).await;

    let res = env
        .post("/problem")
        .user(&admin)
        .json(&json!({
            "body": "some body",
            "solution": 23,
            "image": "image link",
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let id = Uuid::parse_str(res.json::<Value>().await["id"].as_str().unwrap()).unwrap();

    let res = env
        .post("/problem/order")
        .user(&admin)
        .json(&json!({
            "type": "INSERT",
            "id": id,
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let start = Utc::now() + Duration::from_secs(5);
    let start = start - Duration::from_nanos(start.timestamp_subsec_nanos() as u64);

    let res = env
        .patch("/competition/time")
        .user(&admin)
        .json(&json!({
            "start_time": start.timestamp(),
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let owner = env.register_user().await;

    let team = env.create_team(&owner).await;
    team.lock().await;

    let mut socket = env.socket("/ws").start().await;

    socket
        .send(Message::Text(
            json!({"token": owner.access_token().to_owned()}).to_string(),
        ))
        .await
        .unwrap();

    assert_team_info!(socket, owner);

    let message = utils::get_socket_message(socket.next().await);

    let after = Utc::now();

    assert_json_include!(
        actual: message,
        expected: json!({
            "event": "INSERT_PROBLEM",
        }),
    );

    let diff = after - start;

    assert!(diff.num_milliseconds() >= 0);
    assert!(diff.num_milliseconds() < 1000);
}
