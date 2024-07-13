use super::{macros::assert_team_info, setup::Env, user::User};
use http::StatusCode;
use serde_json::json;

#[allow(unused)]
pub struct Team {
    owner: User,
    number: u64,
    env: Env,
}

impl Team {
    pub(super) fn new(env: &Env, owner: User, number: u64) -> Self {
        Team {
            owner,
            number,
            env: env.clone(),
        }
    }

    #[allow(unused)]
    pub async fn get_code(&self) -> String {
        let mut socket = self.env.socket("/ws").start().await;
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
            .env
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
        format!("Test Team {}", self.number)
    }
}
