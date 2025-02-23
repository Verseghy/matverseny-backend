mod m20221125_000213_create_users_table;
mod m20221125_001445_create_teams_table;
mod m20221125_002951_create_team_members_table;
mod m20221208_194301_create_problems_table;
mod m20230107_191407_create_problems_order_table;
mod m20230304_232047_create_times_table;
mod m20230312_051936_create_solutions_history;

pub use sea_orm_migration::MigratorTrait;
use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20221125_000213_create_users_table::Migration),
            Box::new(m20221125_001445_create_teams_table::Migration),
            Box::new(m20221125_002951_create_team_members_table::Migration),
            Box::new(m20221208_194301_create_problems_table::Migration),
            Box::new(m20230107_191407_create_problems_order_table::Migration),
            Box::new(m20230304_232047_create_times_table::Migration),
            Box::new(m20230312_051936_create_solutions_history::Migration),
        ]
    }
}
