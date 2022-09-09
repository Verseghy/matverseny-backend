use crate::utils::{create_table_down, create_table_up};
use entity::{teams, users};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table_up!(manager, teams::Entity);

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("FK_teams_owner")
                    .from(teams::Entity, teams::Column::Owner)
                    .to(users::Entity, users::Column::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("FK_teams_coowner")
                    .from(teams::Entity, teams::Column::Coowner)
                    .to(users::Entity, users::Column::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table_down!(manager, teams::Entity);
        Ok(())
    }
}
