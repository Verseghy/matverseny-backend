use sea_orm::entity::prelude::*;
use uuid::Uuid;

pub mod constraints {
    pub const PK_PROBLEMS_ORDER: &str = "PK_problems_order";
    pub const UC_PROBLEMS_ORDER_NEXT: &str = "UC_problems_order_next";
    pub const FK_PROBLEMS_ORDER_ID: &str = "FK_problems_order_id";
    pub const FK_PROBLEMS_ORDER_NEXT: &str = "FK_problems_order_next";
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "problems_order")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub next: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
