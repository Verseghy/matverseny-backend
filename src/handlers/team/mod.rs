mod create;
mod join;
mod leave;

use crate::shared::SharedTrait;
use axum::{routing::post, Router};

/// Routes for team management
///
/// # Member actions
/// POST  /team/create
/// POST  /team/join
/// POST  /team/leave
/// GET   /team
///
/// # Owner actions
/// PATCH /team
/// POST  /team/disband
/// POST  /team/lock
/// POST  /team/coowner
///
/// # Co-Owner actions
/// POST  /team/kick
/// POST  /team/code
pub fn routes<S: SharedTrait>() -> Router {
    Router::new()
        .route("/create", post(create::create_team::<S>))
        .route("/join", post(join::join_team::<S>))
        .route("/leave", post(leave::leave_team::<S>))
}

#[inline]
pub(super) fn get_kafka_topic(team_id: &str) -> String {
    format!("Team-{}", team_id)
}
