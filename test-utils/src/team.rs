use crate::{User, assert_team_info, prelude::*};

pub struct Team {
    owner: User,
    app: App,
    number: u64,
}

impl Team {
    pub(crate) fn new(owner: User, app: App, number: u64) -> Self {
        Team { owner, app, number }
    }

    pub async fn get_code(&self) -> String {
        let mut socket = self.app.socket("/v1/ws").start().await;
        let message = assert_team_info!(socket, self.owner);

        let code = message["data"]["code"]
            .as_str()
            .expect("no code")
            .to_owned();

        let _ = socket.close(None).await;

        code
    }

    pub async fn lock(&self) {
        let res = self
            .app
            .patch("/v1/team")
            .user(&self.owner)
            .json(&json!({
                "locked": true,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    pub fn get_name(&self) -> String {
        format!("Team-{}", self.number)
    }
}
