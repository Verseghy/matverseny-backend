use sea_orm_migration::prelude::*;
use entity::{users, teams};

const FOREIGN_KEY: &str = "FK_users_team";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FOREIGN_KEY)
                    .from(users::Entity, users::Column::Team)
                    .to(teams::Entity, teams::Column::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(FOREIGN_KEY)
                    .table(users::Entity)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
