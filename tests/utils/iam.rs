use super::{setup::Env, user::UserLike, uuid};
use libiam::testing::actions::assign_action_to_user;

#[derive(Debug)]
pub struct User {
    pub email: String,
    user: libiam::User,
}

impl UserLike for User {
    fn access_token(&self) -> &str {
        self.user.token()
    }

    fn id(&self) -> String {
        self.user.id().to_string()
    }
}

pub async fn register_user(env: &Env) -> User {
    let email = format!("{}@test.test", uuid());

    let user = libiam::User::register(&env.iam, "Test User", &email, "password")
        .await
        .unwrap();

    User { email, user }
}

#[allow(unused)]
pub async fn make_admin(env: &Env, user: &impl UserLike) {
    tracing::trace!("making user '{}' admin", user.id());
    assign_action_to_user(&env.iam_db, "mathcompetition.problems", &user.id()).await;
    assign_action_to_user(&env.iam_db, "mathcompetition.admin", &user.id()).await;
}
