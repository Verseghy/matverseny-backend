use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "teams")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub name: String,
    pub owner: String,
    pub coowner: Option<String>,
    pub locked: bool,
    #[sea_orm(unique)]
    pub join_code: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub fn find_by_join_code(code: &str) -> Select<Entity> {
        Self::find().filter(Column::JoinCode.eq(code))
    }
}
