use super::setup::Env;
use http::StatusCode;
use serde_json::json;

pub trait UserLike {
    fn access_token(&self) -> &str;
    fn id(&self) -> String;
}

#[allow(unused)]
#[derive(Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub access_token: String,
    env: Env,
}

impl User {
    pub(super) fn new(id: String, email: String, access_token: String, env: Env) -> Self {
        User {
            id,
            email,
            access_token,
            env,
        }
    }

    #[allow(unused)]
    pub async fn join(&self, code: &str) {
        let res = self
            .env
            .post("/team/join")
            .user(self)
            .json(&json!({
                "code": code,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::OK);
    }
}

impl UserLike for User {
    fn access_token(&self) -> &str {
        &self.access_token
    }

    fn id(&self) -> String {
        self.id.clone()
    }
}
