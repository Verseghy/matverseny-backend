use crate::team_members;

use super::users;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "teams")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    pub owner: Uuid,
    pub co_owner: Option<Uuid>,
    pub locked: bool,
    #[sea_orm(unique)]
    pub join_code: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Owner,
    Coowner,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Owner => Entity::belongs_to(users::Entity)
                .from(Column::Owner)
                .to(users::Column::Id)
                .into(),
            Self::Coowner => Entity::belongs_to(users::Entity)
                .from(Column::CoOwner)
                .to(users::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<users::Entity> for Entity {
    fn to() -> RelationDef {
        team_members::Relation::User.def()
    }

    fn via() -> Option<RelationDef> {
        Some(team_members::Relation::Team.def().rev())
    }
}

impl Entity {
    #[inline]
    pub fn find_by_join_code(code: &str) -> Select<Entity> {
        Self::find().filter(Column::JoinCode.eq(code))
    }

    #[inline]
    pub fn find_from_member(user_id: &Uuid) -> Select<Entity> {
        users::Entity::find_related().filter(users::Column::Id.eq(*user_id))
    }
}
