use super::teams;
use sea_orm::entity::prelude::*;
// wtf?
use sea_orm::sea_query;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub school: String,
    pub class: Class,
    pub team: Option<String>,
}

#[derive(
    EnumIter, DeriveActiveEnum, PartialEq, Eq, Clone, Debug, Serialize_repr, Deserialize_repr,
)]
#[repr(i16)]
#[sea_orm(rs_type = "i16", db_type = "TinyInteger")]
pub enum Class {
    Nine = 9,
    Ten = 10,
    Eleven = 11,
    Twelve = 12,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<teams::Entity> for Entity {
    fn to() -> RelationDef {
        Entity::belongs_to(teams::Entity)
            .from(Column::Team)
            .to(teams::Column::Id)
            .into()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    #[inline]
    pub fn select_team(user_id: &str) -> Select<teams::Entity> {
        Self::find_related().filter(Column::Id.eq(user_id))
    }
}
