use dotenvy::dotenv;
use libiam::testing::{Database, actions::assign_action_to_app, apps::create_app};
use std::env::args;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().ok();

    let name = args().nth(1).expect("no name given");
    let database = Database::connect("postgres://iam:secret@localhost:3306/iam").await;

    let (id, secret) = create_app(&database, &name).await;

    assign_action_to_app(&database, "iam.policy.assign", &id.to_string()).await;
    assign_action_to_app(&database, "iam.user.get", &id.to_string()).await;

    println!("id: {}", id);
    println!("secret: {}", secret);
}
