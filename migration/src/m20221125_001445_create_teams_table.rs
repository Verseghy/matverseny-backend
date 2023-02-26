use entity::{
    teams::{self, constrains::*},
    users,
};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(teams::Entity)
                    .if_not_exists()
                    .col(ColumnDef::new(teams::Column::Id).uuid().not_null())
                    .col(
                        ColumnDef::new(teams::Column::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(teams::Column::Owner).uuid().not_null())
                    .col(ColumnDef::new(teams::Column::CoOwner).uuid().null())
                    .col(ColumnDef::new(teams::Column::Locked).boolean().not_null())
                    .col(
                        ColumnDef::new(teams::Column::JoinCode)
                            .string_len(6)
                            .not_null(),
                    )
                    .primary_key(Index::create().name(PK_TEAMS).col(teams::Column::Id))
                    .index(
                        Index::create()
                            .name(UC_TEAMS_NAME)
                            .col(teams::Column::Name)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name(UC_TEAMS_JOIN_CODE)
                            .col(teams::Column::JoinCode)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_TEAMS_OWNER)
                            .from(teams::Entity, teams::Column::Owner)
                            .to(users::Entity, users::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_TEAMS_CO_OWNER)
                            .from(teams::Entity, teams::Column::CoOwner)
                            .to(users::Entity, users::Column::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(teams::Entity).to_owned())
            .await
    }
}
