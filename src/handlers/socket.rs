use crate::{iam::Claims, Error, Result, SharedTrait};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use entity::{
    teams,
    users::{self, Class},
};
use futures::StreamExt;
use rdkafka::{
    consumer::Consumer, consumer::StreamConsumer, ClientConfig, Message as _, TopicPartitionList,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Rank {
    Owner,
    CoOwner,
    Member,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    id: String,
    name: String,
    class: Class,
    rank: Rank,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    JoinTeam {
        user: String,
    },
    TeamInfo {
        #[serde(skip)]
        id: String,
        name: String,
        code: String,
        locked: bool,
        members: Vec<Member>,
    },
}

pub async fn ws_handler<S: SharedTrait>(
    Extension(shared): Extension<S>,
    claims: Claims,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    tracing::debug!("ws connection");

    ws.on_upgrade(|socket: WebSocket| async move {
        if let Err(err) = handler(&shared, &claims, socket).await {
            tracing::error!("socket failed with: {:?}", err);
        }
    })
}

async fn get_initial_team_info<S: SharedTrait>(
    shared: &S,
    user_id: &str,
) -> Result<Option<(teams::Model, Vec<Member>)>> {
    let result = users::Entity::select_team(user_id).one(shared.db()).await?;

    if let Some(result) = result {
        tracing::debug!("found team");

        let members = teams::Entity::select_users(&result.id)
            .all(shared.db())
            .await?
            .into_iter()
            .map(|user| Member {
                class: user.class,
                rank: {
                    if user.id == result.owner {
                        Rank::Owner
                    // NOTE: use `Option::is_some_and` when it gets stabilized (#93050)
                    } else if matches!(&result.coowner, Some(coowner) if coowner.as_str() == user.id) {
                        Rank::CoOwner
                    } else {
                        Rank::Member
                    }
                },
                id: user.id.clone(),
                // TODO: get the actual name of the user
                name: user.id,
            })
            .collect();

        Ok(Some((result, members)))
    } else {
        tracing::debug!("didn't found team");
        Ok(None)
    }
}

async fn handler<S: SharedTrait>(shared: &S, claims: &Claims, mut socket: WebSocket) -> Result<()> {
    let info = get_initial_team_info(shared, &claims.subject).await?;

    if let Some((team, members)) = info {
        tracing::debug!("got team info: {:?}, members: {:?}", team, members);

        socket
            .send(Message::Text(
                serde_json::to_string(&Event::TeamInfo {
                    id: team.id.clone(),
                    name: team.name,
                    code: team.join_code,
                    locked: team.locked,
                    members,
                })
                .unwrap(),
            ))
            .await
            .map_err(Error::internal)?;

        let bootstrap_servers =
            std::env::var("KAFKA_BOOTSTRAP_SERVERS").map_err(Error::internal)?;

        // TODO: create a global singleton consumer for performance reasons
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", "socket")
            .create()
            .map_err(Error::internal)?;

        consumer
            .assign(&{
                let mut list = TopicPartitionList::new();
                list.add_partition(
                    &crate::handlers::team::get_kafka_topic(&team.id),
                    // TODO: research if this is reliable
                    0,
                );
                list
            })
            // This shouldn't happend because creating team should also create the kafka topic
            .map_err(Error::internal)?;

        let mut stream = consumer.stream();

        loop {
            if let Some(message) = stream.next().await {
                let payload = message
                    .map_err(Error::internal)?
                    .payload()
                    // SAFETY: the backend will always send a payload
                    // NOTE: this could be handled without panicing
                    .expect("no payload")
                    .to_vec();

                // SAFETY: the backend will always send valid utf-8
                let payload = unsafe { String::from_utf8_unchecked(payload) };

                tracing::debug!("event: {:?}", payload);

                socket
                    .send(Message::Text(payload))
                    .await
                    .map_err(Error::internal)?;
            } else {
                // kafka connection closed
                socket.close().await.map_err(Error::internal)?;
                break Ok(());
            }
        }
    } else {
        // If the user is not in a team, then close the websocket, because there won't be any
        // message sent on it.
        socket.close().await.map_err(Error::internal)?;
        Ok(())
    }
}
