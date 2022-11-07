mod utils;

use tokio_tungstenite::tungstenite::Error;
use utils::prelude::*;

#[tokio::test]
async fn no_team() {
    let app = App::new().await;
    let user = app.register_user().await;

    let request = app
        .socket("/ws")
        .user(&user)
        .into_inner()
        .body(())
        .expect("failed to create request");

    let socket = tokio_tungstenite::connect_async(request).await;

    if let Err(Error::Http(response)) = socket {
        assert_eq!(response.status(), error::USER_NOT_IN_TEAM.status());
        // TODO: verify body when https://github.com/snapview/tungstenite-rs/pull/298 lands
        // this is should break when the PR gets merged
        assert_eq!(response.body(), &None);
    } else {
        unreachable!();
    }
}

#[tokio::test]
async fn team_info() {
    let app = App::new().await;
    let user = app.register_user().await;

    let _team = app.create_team(&user).await;

    let mut socket = app.socket("/ws").user(&user).start().await;
    let message = utils::get_socket_message(socket.next().await);

    assert_json_include!(
        actual: message,
        expected: json!({
            "event": "TEAM_INFO",
            "data": {
                "name": "Team-0",
                "members": [{
                    "class": 9,
                    "id": user.id,
                    "rank": "Owner",
                }],
                "locked": false,
            },
        })
    );

    assert!(message["data"].get("code").is_some());
    // TODO: this should equal to the name in the iam
    assert!(message["data"]["members"][0].get("name").is_some());
}
