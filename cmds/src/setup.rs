use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{ObjectMeta, PostParams},
    Api, Client,
};
use libiam::testing::{actions::assign_action_to_app, apps::create_app, Database};
use std::{collections::BTreeMap, env};

const NAME: &str = "matverseny-app";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let client = Client::try_default().await?;
    let secrets: Api<Secret> = Api::default_namespaced(client);

    let secret = secrets.get_opt(NAME).await?;

    if secret.is_some() {
        println!("Secret already exists. Exiting...");
        return Ok(());
    }

    let database = Database::connect(&database_url).await;

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
