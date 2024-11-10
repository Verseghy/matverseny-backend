use crate::UserLike;
use libiam::{testing::actions::assign_action_to_user, Iam};
use std::{
    env,
    sync::{
        atomic::{AtomicU64, Ordering},
        LazyLock, OnceLock,
    },
};
use uuid::Uuid;

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

static TEST_ID: LazyLock<String> = LazyLock::new(|| {
    let id = Uuid::new_v4();
    id.as_hyphenated()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
});

static USER_COUNT: AtomicU64 = AtomicU64::new(0);
static USER_PASSWORD: &str = "test";

pub(crate) fn get_iam() -> &'static Iam {
    static INIT: OnceLock<Iam> = OnceLock::new();
    INIT.get_or_init(|| Iam::new(&env::var("IAM_URL").expect("IAM_URL not set")))
}

pub async fn get_db() -> &'static libiam::testing::Database {
    static DB: tokio::sync::OnceCell<libiam::testing::Database> =
        tokio::sync::OnceCell::const_new();

    DB.get_or_init(|| async {
        libiam::testing::Database::connect("mysql://iam:secret@127.0.0.1:3306/iam").await
    })
    .await
}

pub async fn register_user() -> User {
    let email = format!(
        "TestUser-{}-{}@test.test",
        *TEST_ID,
        USER_COUNT.fetch_add(1, Ordering::Relaxed)
    );

    let iam = get_iam();

    let user = libiam::User::register(iam, "Test User", &email, USER_PASSWORD)
        .await
        .unwrap();

    User { email, user }
}

pub async fn make_admin(user: &impl UserLike) {
    let db = get_db().await;
    tracing::trace!("making user '{}' admin", user.id());
    assign_action_to_user(db, "mathcompetition.problems", &user.id()).await;
    assign_action_to_user(db, "mathcompetition.admin", &user.id()).await;
}
