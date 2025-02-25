use uuid::{Uuid, fmt::Simple};

#[inline(always)]
pub fn team_info(team_id: &Uuid) -> String {
    let mut buf = [0u8; Simple::LENGTH];
    let id = team_id.as_simple().encode_lower(&mut buf);
    format!("team.{id}.info")
}

#[inline(always)]
pub fn team_solutions(team_id: &Uuid) -> String {
    let mut buf = [0u8; Simple::LENGTH];
    let id = team_id.as_simple().encode_lower(&mut buf);
    format!("team.{id}.solutions")
}

#[inline(always)]
pub fn problems() -> &'static str {
    "info.problems"
}

#[inline(always)]
pub const fn times() -> &'static str {
    "info.times"
}
