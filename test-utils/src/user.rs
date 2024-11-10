use crate::prelude::*;

#[derive(Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub access_token: String,
    app: App,
}

impl User {
    pub(crate) fn new(id: String, email: String, access_token: String, app: App) -> Self {
        User {
            id,
            email,
            access_token,
            app,
        }
    }

    pub async fn join(&self, code: &str) {
        let res = self
            .app
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

pub trait UserLike {
    fn access_token(&self) -> &str;
    fn id(&self) -> String;
}

impl UserLike for User {
    fn access_token(&self) -> &str {
        &self.access_token
    }

    fn id(&self) -> String {
        self.id.clone()
    }
}
