use dotenvy::dotenv;
use libiam::testing::{apps::create_app, Database};
use std::env::args;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().ok();

    let name = args().nth(1).expect("no name given");
    let database = Database::connect("mysql://iam:secret@localhost:3306/iam").await;

    let (id, secret) = create_app(&database, &name).await;

    println!("id: {}", id.to_string());
    println!("secret: {}", secret);
}
