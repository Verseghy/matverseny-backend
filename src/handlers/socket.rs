use crate::{
    error,
    iam::{Claims, IamTrait},
    utils::topics,
    Result, StateTrait,
};
use axum::{
    extract::{
        ws::{close_code, CloseFrame, Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use bytes::Buf;
use entity::{
    teams,
    users::{self, Class},
};
use futures::StreamExt;
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    ClientConfig, Message as _, TopicPartitionList,
};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, error::Error as _, mem::MaybeUninit, time::Duration};
use tokio::time::{self, timeout};
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum Rank {
    Owner,
    CoOwner,
    Member,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    id: Uuid,
    name: String,
    class: Class,
    rank: Rank,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    JoinTeam {
        user: Uuid,
        name: String,
    },
    LeaveTeam {
        user: Uuid,
    },
    TeamInfo {
        #[serde(skip)]
        id: Uuid,
        name: String,
        code: String,
        locked: bool,
        members: Vec<Member>,
    },
    UpdateTeam {
        name: Option<String>,
        owner: Option<Uuid>,
        #[serde(default, with = "::serde_with::rust::double_option")]
        co_owner: Option<Option<Uuid>>,
        locked: Option<bool>,
        code: Option<String>,
    },
    DisbandTeam,
    KickUser {
        user: Uuid,
    },
    UpdateTime {
        start_time: Option<i64>,
        end_time: Option<i64>,
    },
}

pub async fn ws_handler<S: StateTrait>(
    State(state): State<S>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|mut socket: WebSocket| async move {
        if let Err(err) = socket_handler(state, &mut socket).await {
            let error_bytes = err.to_bytes();
            let error_text = std::str::from_utf8(error_bytes.chunk()).unwrap();

            // it's okay to ignore the error here
            let _ = socket
                .send(Message::Close(Some(CloseFrame {
                    code: close_code::ERROR,
                    // TODO: we copy here because the `reason` field neeeds static life time, but
                    // actually it is okay to drop the value after the future finishes
                    reason: Cow::Owned(error_text.to_owned()),
                })))
                .await;

            socket.next().await;
            warn!("socket ended with error: {:?}", err);
        } else {
            // info!("socket ended");
        }
    })
}

async fn socket_handler<S: StateTrait>(state: S, socket: &mut WebSocket) -> Result<()> {
    let (team, members, claims) = socket_auth(&state, socket).await?;
    let claims_span = info_span!("claims", user_id = claims.subject.to_string());

    async move {
        let consumer = create_consumer(&team.id).await?;

        socket
            .send(Message::Text(
                serde_json::to_string(&Event::TeamInfo {
                    id: team.id,
                    name: team.name,
                    code: team.join_code,
                    locked: team.locked,
                    members,
                })
                .unwrap(),
            ))
            .await
            .map_err(|err| {
                error!("websocket error: {:?}", err);
                error::WEBSOCKET_ERROR
            })?;

        let mut kafka_stream = consumer.stream();

        loop {
            tokio::select! {
                message = timeout(Duration::from_secs(5), kafka_stream.next()) => {
                    let Ok(message) = message else {
                        debug!("timeout");
                        continue;
                    };

                    let Some(message) = message else {
                        error!("kafka stream closed unexpectedly");
                        break Err(error::INTERNAL)
                    };

                    let message = message?;

                    debug!("kafka message: {:?}", message);

                    let Some(payload) = message.payload() else {
                        warn!("got kafka message without payload");
                        // This shouldn't happen so if somehow it still happens just ignore it
                        continue
                    };

                    // SAFETY: the backend will always send valid utf-8
                    let payload = unsafe { std::str::from_utf8_unchecked(payload) };
                    let event = serde_json::from_str(payload)?;

                    if matches!(event, Event::DisbandTeam)
                        || matches!(event, Event::KickUser { user } if user == claims.subject)
                    {
                        let _ = socket.send(Message::Close(Some(CloseFrame {
                            code: close_code::NORMAL,
                            reason: Cow::Owned(payload.to_owned()),
                        }))).await;

                        socket.next().await;

                        return Ok(())
                    }

                    if let Err(err) = socket.send(Message::Text(payload.to_owned())).await {
                        let tungstenite_error = err.source().unwrap().downcast_ref::<TungsteniteError>().unwrap();
                        error!("websocket error: {:?}", tungstenite_error);
                        break Err(error::WEBSOCKET_ERROR)
                    }
                }
                message = socket.next() => {
                    match message {
                        Some(Ok(Message::Close(_))) | None => break Ok(()),
                        Some(Ok(_)) => {
                            warn!("got message on websocket");
                            continue
                        }
                        Some(Err(err)) => {
                            error!("websocket error: {:?}", err);
                            return Err(error::WEBSOCKET_ERROR)
                        },
                    };
                }
            }
        }
    }
        .instrument(claims_span)
        .await
}

type TeamInfo = (teams::Model, Vec<Member>, Claims);

#[derive(Serialize, Deserialize)]
struct TokenJSON {
    token: String,
}

async fn socket_auth<S: StateTrait>(state: &S, socket: &mut WebSocket) -> Result<TeamInfo> {
    let message = {
        let timeout = time::sleep(Duration::from_secs(10));
        tokio::pin!(timeout);

        let mut uninit = MaybeUninit::uninit();

        tokio::select! {
            message = socket.next() => {
                match message {
                    None => {
                        error!("websocket stream closed unexpectedly");
                        // The error doesn't matter because the socket is already closed
                        return Err(error::INTERNAL);
                    },
                    Some(Ok(msg)) => uninit.write(msg),
                    Some(Err(err)) => {
                        error!("websocket error: {:?}", err);
                        return Err(error::WEBSOCKET_ERROR)
                    },
                };
            },
            _ = &mut timeout => {
                return Err(error::WEBSOCKET_AUTH_TIMEOUT);
            },
        };

        // SAFETY: this is initialized because if there is no message then it will return early
        unsafe { uninit.assume_init() }
    };

    let token_str = match message {
        Message::Text(t) => t,
        _ => return Err(error::WEBSOCKET_WRONG_MESSAGE_TYPE),
    };

    let token_json: TokenJSON =
        serde_json::from_str(&token_str).map_err(|_| error::JWT_INVALID_TOKEN)?;

    let claims = state.iam().get_claims(&token_json.token)?;

    let user = users::Entity::find_by_id(claims.subject)
        .one(state.db())
        .await?
        .ok_or(error::USER_NOT_REGISTERED)?;

    let result = teams::Entity::find_from_member(&user.id)
        .one(state.db())
        .await?
        .ok_or(error::USER_NOT_IN_TEAM)?;

    let raw_members = users::Entity::find_in_team(&result.id)
        .all(state.db())
        .await?;

    let mut members = Vec::with_capacity(raw_members.len());

    for member in raw_members {
        let name = state
            .iam_app()
            .get_user_info(&format!("UserID-{}", &member.id))
            .await
            .map_err(|error| {
                error!("iam error: {:?}", error);
                error::IAM_FAILED_GET_NAME
            })?
            .name;
        let rank = if member.id == result.owner {
            Rank::Owner
        // NOTE: use `Option::is_some_and` when it gets stabilized (#93050)
        } else if matches!(&result.co_owner, Some(co_owner) if *co_owner == member.id) {
            Rank::CoOwner
        } else {
            Rank::Member
        };

        members.push(Member {
            class: member.class,
            rank,
            id: member.id,
            name,
        })
    }

    Ok((result, members, claims))
}

// TODO: create a global singleton consumer for performance reasons
async fn create_consumer(team_id: &Uuid) -> Result<StreamConsumer> {
    let bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
        .expect("environment variable KAFKA_BOOTSTRAP_SERVERS is not set");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", bootstrap_servers)
        .set("group.id", "matverseny-backend")
        .set("enable.partition.eof", "false")
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "latest")
        .create()?;

    consumer.assign(&{
        let mut list = TopicPartitionList::new();
        list.add_partition(&topics::team_info(team_id), 0);
        list.add_partition(topics::times(), 0);
        list
    })?;

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    Ok(consumer)
}
