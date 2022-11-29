use super::{team_members, teams};
use sea_orm::entity::prelude::*;
use serde_repr::{Deserialize_repr, Serialize_repr};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub school: String,
    pub class: Class,
}

#[derive(
    EnumIter, DeriveActiveEnum, PartialEq, Eq, Clone, Debug, Serialize_repr, Deserialize_repr,
)]
#[repr(i16)]
#[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
pub enum Class {
    Nine = 9,
    Ten = 10,
    Eleven = 11,
    Twelve = 12,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Related<teams::Entity> for Entity {
    fn to() -> RelationDef {
        team_members::Relation::Team.def()
    }

    fn via() -> Option<RelationDef> {
        Some(team_members::Relation::User.def().rev())
    }
}

impl Entity {
    #[inline]
    pub fn find_in_team(team_id: &Uuid) -> Select<Entity> {
        teams::Entity::find_related().filter(teams::Column::Id.eq(*team_id))
    }
}
