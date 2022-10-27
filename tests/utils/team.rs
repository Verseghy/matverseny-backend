use super::prelude::*;
use super::{get_socket_message, User};

#[allow(unused)]
pub struct Team {
    id: String,
    owner: User,
    app: App,
}

impl Team {
    pub(super) fn new(id: String, owner: User, app: App) -> Self {
        Team { id, owner, app }
    }

    #[allow(unused)]
    pub async fn get_code(&self) -> String {
        let mut socket = self.app.socket("/ws").user(&self.owner).start().await;
        let message = get_socket_message(socket.next().await);

        assert!(message.is_object());
        assert!(message["event"].is_string());
        assert_eq!(message["event"].as_str().unwrap(), "TEAM_INFO");

        let code = message["data"]["code"]
            .as_str()
            .expect("no code")
            .to_owned();

        socket.close(None).await;

        code
    }

    #[allow(unused)]
    pub async fn lock(&self) {
        let res = self
            .app
            .patch("/team")
            .user(&self.owner)
            .json(&json!({
                "locked": true,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }
}
