use super::{teams, users};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "team_members")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub team_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    User,
    Team,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::User => Entity::belongs_to(users::Entity)
                .from(Column::UserId)
                .to(users::Column::Id)
                .into(),
            Self::Team => Entity::belongs_to(teams::Entity)
                .from(Column::TeamId)
                .to(teams::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
