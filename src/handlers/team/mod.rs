mod code;
mod create;
mod disband;
mod join;
mod kick;
mod leave;
mod update;

use crate::state::StateTrait;
use axum::{
    routing::{patch, post},
    Router,
};

/// Routes for team management
///
/// # Member actions
/// POST  /team/create
/// POST  /team/join
/// POST  /team/leave
///
/// # Owner actions
/// PATCH /team
/// POST  /team/disband
///
/// # Co-Owner actions
/// POST  /team/kick
/// POST  /team/code
///
/// # Admin actions
/// GET /team/:id
/// GET /teams
pub fn routes<S: StateTrait>() -> Router<S> {
    Router::new()
        .route("/create", post(create::create_team::<S>))
        .route("/join", post(join::join_team::<S>))
        .route("/leave", post(leave::leave_team::<S>))
        .route("/", patch(update::update_team::<S>))
        .route("/disband", post(disband::disband_team::<S>))
        .route("/kick", post(kick::kick_user::<S>))
        .route("/code", post(code::regenerate_code::<S>))
}
