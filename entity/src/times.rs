use sea_orm::entity::prelude::*;

pub mod constrains {
    pub const PK_TIMES: &str = "PK_times";
}

pub mod constants {
    pub const START_TIME: &str = "start_time";
    pub const END_TIME: &str = "end_time";
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "times")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub name: String,
    pub time: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub fn find_start_time() -> Select<Entity> {
        Entity::find_by_id(constants::START_TIME)
    }

    pub fn find_end_time() -> Select<Entity> {
        Entity::find_by_id(constants::END_TIME)
    }
}
