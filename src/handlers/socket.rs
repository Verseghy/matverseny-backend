use crate::{error, iam::Claims, Error, Result, SharedTrait};
use axum::{
    extract::ws::{close_code, CloseFrame, Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use entity::{
    teams,
    users::{self, Class},
};
use futures::StreamExt;
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    ClientConfig, Message as _, TopicPartitionList,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, error::Error as _};
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;

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

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    JoinTeam {
        user: String,
    },
    LeaveTeam {
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
    UpdateTeam {
        name: Option<String>,
        owner: Option<String>,
        #[serde(default, with = "::serde_with::rust::double_option")]
        coowner: Option<Option<String>>,
        locked: Option<bool>,
        code: Option<String>,
    },
    DisbandTeam,
    KickUser {
        user: String,
    },
}

pub async fn ws_handler<S: SharedTrait>(
    Extension(shared): Extension<S>,
    claims: Claims,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse> {
    tracing::debug!("ws connection");

    if let Some(team_info) = get_initial_team_info(&shared, &claims.subject).await? {
        let consumer = create_consumer(&team_info.0.id)?;

        // TODO: this seem to work reliably, but it is not a bullet proof solution
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(ws.on_upgrade(move |socket: WebSocket| async move {
            if let Err(err) = handler(socket, consumer, claims, team_info).await {
                tracing::error!("socket failed with: {:?}", err);
            }
        }))
    } else {
        tracing::warn!("started websocket without a team");
        Err(error::USER_NOT_IN_TEAM)
    }
}

// TODO: create a global singleton consumer for performance reasons
fn create_consumer(team_id: &str) -> Result<StreamConsumer> {
    let bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS").map_err(Error::internal)?;

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", bootstrap_servers)
        .set("group.id", "socket")
        .create()
        .map_err(Error::internal)?;

    consumer
        .assign(&{
            let mut list = TopicPartitionList::new();
            list.add_partition(
                &crate::handlers::team::get_kafka_topic(team_id),
                // TODO: research if this is reliable
                0,
            );
            list
        })
        // This shouldn't happend because creating team should also create the kafka topic
        .map_err(Error::internal)?;

    Ok(consumer)
}

type TeamInfo = (teams::Model, Vec<Member>);

async fn get_initial_team_info<S: SharedTrait>(
    shared: &S,
    user_id: &str,
) -> Result<Option<TeamInfo>> {
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

async fn handler(
    mut socket: WebSocket,
    consumer: StreamConsumer,
    claims: Claims,
    (team, members): TeamInfo,
) -> Result<()> {
    let mut stream = consumer.stream();

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

    loop {
        tokio::select! {
            message = stream.next() => {
                if let Some(message) = message {
                    let payload = message
                        .map_err(Error::internal)?
                        .payload()
                        // SAFETY: the backend will always send a payload
                        // NOTE: this could be handled without panicing
                        .expect("no payload")
                        .to_vec();

                    // SAFETY: the backend will always send valid utf-8
                    let payload = unsafe { String::from_utf8_unchecked(payload) };

                    let event = serde_json::from_str(&payload).unwrap();

                    tracing::debug!("subject: {}, event: {:?}", claims.subject, event);

                    if matches!(event, Event::DisbandTeam)
                        || matches!(event, Event::KickUser { user } if user == claims.subject)
                    {
                        let _ = socket.send(Message::Close(Some(CloseFrame {
                            code: close_code::NORMAL,
                            reason: Cow::Owned(payload),
                        }))).await;

                        return Ok(())
                    }

                    tracing::debug!("event: {:?}", payload);

                    if let Err(err) = socket.send(Message::Text(payload)).await {
                        let tungstenite_error = err.source().unwrap().downcast_ref::<TungsteniteError>().unwrap();
                        tracing::error!("error: {:?}", tungstenite_error);
                        break Err(Error::internal(err))
                    }
                } else {
                    // kafka connection closed
                    socket.close().await.map_err(Error::internal)?;
                    break Ok(());
                }
            }
            message = socket.recv() => {
                if let Some(Ok(Message::Close(_))) = message {
                    tracing::debug!("socket closed by client");
                    break Ok(())
                } else {
                    // socket is closed
                    break Ok(())
                }
            }
        }
    }
}
