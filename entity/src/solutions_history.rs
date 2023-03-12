use sea_orm::entity::prelude::*;
use uuid::Uuid;

pub mod constraints {
    pub const PK_SOLUTIONS_HISTORY: &str = "PK_solutions_history";
    pub const FK_TEAMS: &str = "FK_teams_history";
    pub const FK_PROBLEMS: &str = "FK_problems_history";
    pub const FK_USERS: &str = "FK_users_history";
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "solutions_history")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment=false)]
    pub id: Uuid,
    pub team: Uuid,
    pub problem: Uuid,
    pub user: Uuid,
    pub solution: Option<i64>,
    pub created_at: DateTime
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}