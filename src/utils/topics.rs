use uuid::{fmt::Simple, Uuid};

#[inline(always)]
pub fn team_info(team_id: &Uuid) -> String {
    let mut buf = [0u8; Simple::LENGTH];
    let id = team_id.as_simple().encode_lower(&mut buf);
    format!("team.{id}.info")
}
