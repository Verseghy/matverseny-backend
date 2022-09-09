mod utils;

pub use sea_orm_migration::prelude::*;

mod m20220906_033629_create_users_table;
mod m20220908_120622_create_teams_table;
mod m20220909_185939_create_users_table_foreign_key;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220906_033629_create_users_table::Migration),
            Box::new(m20220908_120622_create_teams_table::Migration),
            Box::new(m20220909_185939_create_users_table_foreign_key::Migration),
        ]
    }
}
