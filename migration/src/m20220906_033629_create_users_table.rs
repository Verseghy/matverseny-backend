use super::utils::{create_table_up, create_table_down};
use sea_orm_migration::prelude::*;
use entity::users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table_up!(manager, users::Entity);
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table_down!(manager, users::Entity);
        Ok(())
    }
}
