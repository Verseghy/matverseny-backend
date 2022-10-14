mod utils;

use tokio_tungstenite::tungstenite::Message;
use utils::prelude::*;

#[tokio::test]
async fn close_when_no_team() {
    let app = App::new().await;
    let user = app.register_user().await;

    let mut socket = app.socket("/ws").user(&user).start().await;

    assert!(matches!(socket.next().await, Some(Ok(Message::Close(_)))));
}

#[tokio::test]
async fn team_info() {
    let app = App::new().await;
    let user = app.register_user().await;

    let _team = app.create_team(&user).await;

    let mut socket = app.socket("/ws").user(&user).start().await;
    let message = utils::get_socket_message(socket.next().await);

    assert!(message.is_object());
    assert!(message["event"].is_string());
    assert_eq!(message["event"].as_str().unwrap(), "TEAM_INFO");
}

#[tokio::test]
async fn join_event() {
    let app = App::new().await;
    let user = app.register_user().await;
    let user2 = app.register_user().await;
    let team = app.create_team(&user).await;

    let mut socket = app.socket("/ws").user(&user).start().await;
    let message = utils::get_socket_message(socket.next().await);

    assert!(message.is_object());
    assert!(message["event"].is_string());
    assert_eq!(message["event"].as_str().unwrap(), "TEAM_INFO");

    user2.join(&team.get_code().await).await;

    let message = socket.next().await;
    let message = utils::get_socket_message(message);

    assert!(message.is_object());
    assert!(message["event"].is_string());
    assert_eq!(message["event"].as_str().unwrap(), "JOIN_TEAM");
}