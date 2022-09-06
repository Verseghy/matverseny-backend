use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub school: String,
    pub class: Class,
}

#[derive(EnumIter, DeriveActiveEnum, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[sea_orm(rs_type = "u8", db_type = "TinyUnsigned")]
pub enum Class {
    Nine = 9,
    Ten = 10,
    Eleven = 11,
    Twelve = 12,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef");
    }
}

impl ActiveModelBehavior for ActiveModel {}
