mod utils;

use std::time::Duration;

use chrono::Utc;
use tokio_tungstenite::tungstenite::Message;
use utils::prelude::*;
use uuid::Uuid;

#[tokio::test]
#[parallel]
async fn timeout() {
    let app = get_cached_app().await;

    let mut socket = app.socket("/ws").start().await;

    assert_close_frame_error!(socket.next().await, error::WEBSOCKET_AUTH_TIMEOUT);
}

#[tokio::test]
#[parallel]
async fn wrong_token() {
    let app = get_cached_app().await;

    let mut socket = app.socket("/ws").start().await;

    socket
        .send(Message::Text("some random invalid token".to_owned()))
        .await
        .unwrap();
    assert_close_frame_error!(socket.next().await, error::JWT_INVALID_TOKEN);
}

#[tokio::test]
#[parallel]
async fn wrong_message_type() {
    let app = get_cached_app().await;

    let mut socket = app.socket("/ws").start().await;

    socket
        .send(Message::Binary(Vec::from("asd".as_bytes())))
        .await
        .unwrap();
    assert_close_frame_error!(socket.next().await, error::WEBSOCKET_WRONG_MESSAGE_TYPE);
}

#[tokio::test]
#[parallel]
async fn user_not_registered() {
    let app = get_cached_app().await;
    let user = utils::iam::register_user().await;

    let mut socket = app.socket("/ws").start().await;

    socket
        .send(Message::Text(
            json!({"token": user.access_token().to_owned()}).to_string(),
        ))
        .await
        .unwrap();
    assert_close_frame_error!(socket.next().await, error::USER_NOT_REGISTERED);
}

#[tokio::test]
#[parallel]
async fn no_team() {
    let app = get_cached_app().await;
    let user = app.register_user().await;

    let mut socket = app.socket("/ws").start().await;

    socket
        .send(Message::Text(
            json!({"token": user.access_token().to_owned()}).to_string(),
        ))
        .await
        .unwrap();
    assert_close_frame_error!(socket.next().await, error::USER_NOT_IN_TEAM);
}

#[tokio::test]
#[parallel]
async fn team_info() {
    let app = get_cached_app().await;
    let user = app.register_user().await;

    let _team = app.create_team(&user).await;

    let mut socket = app.socket("/ws").start().await;
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
                "name": "Team-0",
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
    let user = libiam::testing::users::get_user(utils::iam::get_db().await, &user.id).await;
    assert_eq!(message["data"]["members"][0]["name"], user.name);

    socket.close(None).await.unwrap();
}

#[tokio::test]
#[serial]
async fn dont_send_problems_before_start() {
    let app = get_cached_app().await;
    app.clean_database().await;

    let admin = utils::iam::register_user().await;
    utils::iam::make_admin(&admin).await;

    let res = app
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

    let res = app
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

    let res = app
        .patch("/competition/time")
        .user(&admin)
        .json(&json!({
            "start_time": start.timestamp(),
        }))
        .send()
        .await;

    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let owner = app.register_user().await;

    let team = app.create_team(&owner).await;
    team.lock().await;

    let mut socket = app.socket("/ws").start().await;

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
