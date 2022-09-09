use super::teams;
use sea_orm::entity::prelude::*;
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
#[repr(u8)]
#[sea_orm(rs_type = "u8", db_type = "TinyUnsigned")]
pub enum Class {
    Nine = 9,
    Ten = 10,
    Eleven = 11,
    Twelve = 12,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Team,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::Team => Entity::belongs_to(teams::Entity)
                .from(Column::Team)
                .to(teams::Column::Id)
                .into(),
        }
    }
}

impl Related<teams::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Team.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
