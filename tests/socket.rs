mod utils;

use tokio_tungstenite::tungstenite::Message;
use utils::prelude::*;

#[tokio::test]
async fn timeout() {
    let app = get_cached_app().await;

    let mut socket = app.socket("/ws").start().await;

    assert_close_frame_error!(socket.next().await, error::WEBSOCKET_AUTH_TIMEOUT);
}

#[tokio::test]
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
async fn user_not_registered() {
    let app = get_cached_app().await;
    let user = utils::iam::register_user().await;

    let mut socket = app.socket("/ws").start().await;

    socket
        .send(Message::Text(user.access_token().clone()))
        .await
        .unwrap();
    assert_close_frame_error!(socket.next().await, error::USER_NOT_REGISTERED);
}

#[tokio::test]
async fn no_team() {
    let app = get_cached_app().await;
    let user = app.register_user().await;

    let mut socket = app.socket("/ws").start().await;

    socket
        .send(Message::Text(user.access_token().clone()))
        .await
        .unwrap();
    assert_close_frame_error!(socket.next().await, error::USER_NOT_IN_TEAM);
}

#[tokio::test]
async fn team_info() {
    let app = get_cached_app().await;
    let user = app.register_user().await;

    let _team = app.create_team(&user).await;

    let mut socket = app.socket("/ws").start().await;
    socket
        .send(Message::Text(user.access_token().clone()))
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
    // TODO: this should equal to the name in the iam
    assert!(message["data"]["members"][0].get("name").is_some());

    socket.close(None).await.unwrap();
}
