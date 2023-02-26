use entity::users::{self, constraints::*};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(users::Entity)
                    .if_not_exists()
                    .col(ColumnDef::new(users::Column::Id).uuid().not_null())
                    .col(
                        ColumnDef::new(users::Column::School)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(users::Column::Class)
                            .small_integer()
                            .not_null(),
                    )
                    .primary_key(Index::create().name(PK_USERS).col(users::Column::Id))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(users::Entity).to_owned())
            .await
    }
}
