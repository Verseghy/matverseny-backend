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

async fn get_initial_team_info<S: SharedTrait>(shared: &S, user_id: &str) -> Result<Option<Event>> {
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

        Ok(Some(Event::TeamInfo {
            name: result.name,
            code: result.join_code,
            locked: result.locked,
            members,
        }))
    } else {
        tracing::debug!("didn't found team");
        Ok(None)
    }
}

async fn handler<S: SharedTrait>(shared: &S, claims: &Claims, mut socket: WebSocket) -> Result<()> {
    let info = get_initial_team_info(shared, &claims.subject).await?;

    if let Some(info) = info {
        tracing::debug!("info: {:?}", info);

        socket
            .send(Message::Text(serde_json::to_string(&info).unwrap()))
            .await
            .map_err(Error::internal)?;

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    } else {
        // If the user is not in a team, then close the websocket, because there won't be any
        // message sent on it.
        socket.close().await.map_err(Error::internal)?;
        Ok(())
    }
}
