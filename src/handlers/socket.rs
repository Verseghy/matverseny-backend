use crate::{
    error,
    extractors::UserID,
    utils::{topics, ProblemStream},
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
use chrono::{DateTime, Utc};
use entity::times;
use entity::{
    solutions_history, teams,
    users::{self, Class},
};
use futures::{Stream, StreamExt};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, error::Error as _, mem::MaybeUninit, pin::pin, time::Duration};
use tokio::time::{self, sleep, timeout};
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Rank {
    Owner,
    CoOwner,
    Member,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Member {
    id: Uuid,
    name: String,
    class: Class,
    rank: Rank,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    UpdateTime {
        start_time: Option<i64>,
        end_time: Option<i64>,
    },
    SolutionSet {
        problem: Uuid,
        solution: Option<i64>,
    },
    InsertProblem {
        before: Option<Uuid>,
        id: Uuid,
        body: String,
        image: Option<String>,
    },
    DeleteProblem {
        id: Uuid,
    },
    SwapProblems {
        id1: Uuid,
        id2: Uuid,
    },
    UpdateProblem {
        id: Uuid,
        body: Option<String>,
        image: Option<Option<String>>,
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
    let (team, members, user_id) = socket_auth(&state, socket).await?;
    let claims_span = info_span!("claims", user_id = user_id.to_string());

    async move {
        let mut consumer = std::pin::pin!(create_consumer(&state, &team.id).await?);

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

        let start_time = send_times(&state, socket).await?;

        let mut has_sent_initial_problems = false;

        let mut sleep_until_start = pin!({
            let now = Utc::now();
            let duration = (start_time - now).to_std().unwrap_or(Duration::ZERO);
            sleep(duration)
        });

        let problems = state.problems();
        let mut problems_stream = ProblemStream::new_empty();

        loop {
            tokio::select! {
                _ = &mut sleep_until_start, if !has_sent_initial_problems => {
                    let (mut initial_problems, new_problems_stream) = problems.stream().await;
                    problems_stream = new_problems_stream;

                    while let Some(event) = initial_problems.next().await {
                        let payload = serde_json::to_string(&event).unwrap();
                        if let Err(err) = socket.send(Message::Text(payload)).await {
                            let tungstenite_error = err.source().unwrap().downcast_ref::<TungsteniteError>().unwrap();
                            error!("websocket error: {:?}", tungstenite_error);
                            return Err(error::WEBSOCKET_ERROR)
                        }
                    }

                    send_answers(&state, socket, team.id).await?;

                    has_sent_initial_problems = true;
                }
                problems_event = problems_stream.next(), if has_sent_initial_problems => {
                    let Some(problems_event) = problems_event else {
                        continue
                    };

                    let payload = serde_json::to_string(&problems_event).unwrap();
                    if let Err(err) = socket.send(Message::Text(payload)).await {
                        let tungstenite_error = err.source().unwrap().downcast_ref::<TungsteniteError>().unwrap();
                        error!("websocket error: {:?}", tungstenite_error);
                        break Err(error::WEBSOCKET_ERROR)
                    }
                }
                Ok(Some(event)) = timeout(Duration::from_secs(5), consumer.next()), if has_sent_initial_problems => {
                    let event_text = serde_json::to_string(&event).unwrap();

                    if matches!(event, Event::DisbandTeam)
                        || matches!(event, Event::LeaveTeam { user } if user == *user_id)
                    {
                        let _ = socket.send(Message::Close(Some(CloseFrame {
                            code: close_code::NORMAL,
                            reason: Cow::Owned(event_text),
                        }))).await;

                        socket.next().await;

                        return Ok(())
                    }

                    if let Err(err) = socket.send(Message::Text(event_text)).await {
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

async fn send_times<S: StateTrait>(state: &S, socket: &mut WebSocket) -> Result<DateTime<Utc>> {
    let res = times::Entity::find()
        .filter(
            Condition::any()
                .add(times::Column::Name.eq("start_time"))
                .add(times::Column::Name.eq("end_time")),
        )
        .all(state.db())
        .await?;

    if res.len() != 2 {
        error!("start_time or end_time is not found in the database");
        return Err(error::INTERNAL);
    }

    let start_time = res.iter().find(|i| i.name == "start_time").unwrap();
    let end_time = res.iter().find(|i| i.name == "end_time").unwrap();

    socket
        .send(Message::Text(
            serde_json::to_string(&Event::UpdateTime {
                start_time: Some(start_time.time.timestamp()),
                end_time: Some(end_time.time.timestamp()),
            })
            .unwrap(),
        ))
        .await
        .map_err(|err| {
            error!("websocket error: {:?}", err);
            error::WEBSOCKET_ERROR
        })?;

    Ok(start_time.time)
}

async fn send_answers<S: StateTrait>(
    state: &S,
    socket: &mut WebSocket,
    team_id: Uuid,
) -> Result<()> {
    let res = solutions_history::Entity::find()
        .filter(solutions_history::Column::Team.eq(team_id))
        .distinct_on([solutions_history::Column::Problem])
        .order_by_desc(solutions_history::Column::Problem)
        .order_by_desc(solutions_history::Column::CreatedAt)
        .all(state.db())
        .await?;

    for answer in res {
        let payload = serde_json::to_string(&Event::SolutionSet {
            problem: answer.problem,
            solution: answer.solution,
        })
        .unwrap();
        if let Err(err) = socket.send(Message::Text(payload)).await {
            let tungstenite_error = err
                .source()
                .unwrap()
                .downcast_ref::<TungsteniteError>()
                .unwrap();
            error!("websocket error: {:?}", tungstenite_error);
            return Err(error::WEBSOCKET_ERROR);
        }
    }

    Ok(())
}

type TeamInfo = (teams::Model, Vec<Member>, UserID);

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

    let claims = &state
        .jwt()
        .get_claims(&token_json.token)
        .await
        .map_err(|_| error::INTERNAL)?;

    let user_id = UserID::parse_str(&claims.sub)?;

    let user = users::Entity::find_by_id(*user_id)
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
        } else if result
            .co_owner
            .is_some_and(|co_owner| co_owner == member.id)
        {
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

    Ok((result, members, user_id))
}

async fn create_consumer<'a, 'b, S: StateTrait>(
    state: &'a S,
    team_id: &'a Uuid,
) -> Result<impl Stream<Item = Event> + 'b> {
    let nats = state.nats();

    let combined = futures::stream::select_all([
        nats.subscribe(topics::team_info(team_id)).await?,
        nats.subscribe(topics::team_solutions(team_id)).await?,
        nats.subscribe(topics::times()).await?,
    ]);

    Ok(combined.filter_map(|message| async move {
        serde_json::from_slice::<Event>(&message.payload).ok()
    }))
}
