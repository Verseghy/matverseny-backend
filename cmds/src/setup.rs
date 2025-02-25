use k8s_openapi::api::core::v1::Secret;
use kube::{
    Api, Client,
    api::{ObjectMeta, PostParams},
};
use libiam::{
    Iam, User,
    testing::{
        Database,
        actions::{assign_action_to_app, assign_action_to_user, ensure_action},
        apps::create_app,
    },
};
use rand::distr::{Alphanumeric, SampleString};
use std::{collections::BTreeMap, env};

const NAME: &str = "matverseny-app";

async fn iam_secret(client: Client, database: Database) -> Result<(), Box<dyn std::error::Error>> {
    let secrets: Api<Secret> = Api::default_namespaced(client);

    let secret = secrets.get_opt(NAME).await?;

    if secret.is_some() {
        println!("Secret already exists. Exiting...");
        return Ok(());
    }

    let (id, app_secret) = create_app(&database, "matverseny").await;

    assign_action_to_app(&database, "iam.policy.assign", &id.to_string()).await;
    assign_action_to_app(&database, "iam.user.get", &id.to_string()).await;

    secrets
        .create(
            &PostParams::default(),
            &Secret {
                metadata: ObjectMeta {
                    name: Some(NAME.to_owned()),
                    ..Default::default()
                },
                string_data: Some({
                    let mut map = BTreeMap::new();
                    map.insert("iam_secret".to_owned(), app_secret);
                    map
                }),
                ..Default::default()
            },
        )
        .await?;

    Ok(())
}

async fn admin_user(client: Client, database: Database) -> Result<(), Box<dyn std::error::Error>> {
    let secrets: Api<Secret> = Api::default_namespaced(client);

    let secret = secrets.get_opt("matverseny-admin-user").await?;

    if secret.is_some() {
        println!("Secret already exists. Exiting...");
        return Ok(());
    }

    let iam_url = env::var("IAM_URL").expect("IAM_URL is not set");

    let iam = Iam::new(&iam_url);
    let admin_password = Alphanumeric.sample_string(&mut rand::rng(), 64);
    let user = User::register(&iam, "admin", "matverseny@admin.admin", &admin_password).await?;

    ensure_action(&database, "mathcompetition.admin", true).await;
    ensure_action(&database, "mathcompetition.problems", true).await;
    assign_action_to_user(&database, "iam.user.list", &user.id().to_string()).await;
    assign_action_to_user(&database, "iam.user.get", &user.id().to_string()).await;
    assign_action_to_user(&database, "mathcompetition.admin", &user.id().to_string()).await;
    assign_action_to_user(
        &database,
        "mathcompetition.problems",
        &user.id().to_string(),
    )
    .await;

    secrets
        .create(
            &PostParams::default(),
            &Secret {
                metadata: ObjectMeta {
                    name: Some("matverseny-admin-user".to_owned()),
                    ..Default::default()
                },
                string_data: Some({
                    let mut map = BTreeMap::new();
                    map.insert("email".to_owned(), "matverseny@admin.admin".to_owned());
                    map.insert("password".to_owned(), admin_password);
                    map
                }),
                ..Default::default()
            },
        )
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let database = Database::connect(&database_url).await;

    let client = Client::try_default().await?;

    iam_secret(client.clone(), database.clone()).await?;
    admin_user(client, database).await?;

    Ok(())
}
