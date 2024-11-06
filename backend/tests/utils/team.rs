use super::prelude::*;
use super::{super::utils, User};

#[allow(unused)]
pub struct Team {
    owner: User,
    app: App,
    number: u64,
}

impl Team {
    pub(super) fn new(owner: User, app: App, number: u64) -> Self {
        Team { owner, app, number }
    }

    #[allow(unused)]
    pub async fn get_code(&self) -> String {
        let mut socket = self.app.socket("/ws").start().await;
        let message = assert_team_info!(socket, self.owner);

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

    #[allow(unused)]
    pub fn get_name(&self) -> String {
        format!("Team-{}", self.number)
    }
}
