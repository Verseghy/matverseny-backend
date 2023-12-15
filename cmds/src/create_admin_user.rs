use dotenvy::dotenv;
use libiam::testing::actions::ensure_action;
use libiam::{
    testing::{actions::assign_action_to_user, Database},
    Iam, User,
};
use std::env::{self, args};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().ok();

    let email = args().nth(1).expect("no email given");
    let iam = Iam::new(&env::var("IAM_URL").expect("IAM_URL is not set"));
    let database = Database::connect("mysql://iam:secret@localhost:3306/iam").await;

    let id = 'id: {
        if let Ok(user) = User::login(&iam, &email, "test").await {
            break 'id *user.id();
        }

        *User::register(&iam, "Admin User", &email, "test")
            .await
            .unwrap()
            .id()
    };

    ensure_action(&database, "mathcompetition.problems", false).await;
    ensure_action(&database, "mathcompetition.admin", false).await;
    assign_action_to_user(&database, "mathcompetition.problems", &id.to_string()).await;
    assign_action_to_user(&database, "mathcompetition.admin", &id.to_string()).await;
    assign_action_to_user(&database, "iam.policy.assign", &id.to_string()).await;
    assign_action_to_user(&database, "iam.user.list", &id.to_string()).await;

    let user = User::login(&iam, &email, "test").await.unwrap();

    println!("{}", user.token());
}
