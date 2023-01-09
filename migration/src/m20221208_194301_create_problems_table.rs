use entity::problems;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(problems::Entity)
                    .if_not_exists()
                    .col(ColumnDef::new(problems::Column::Id).uuid().not_null())
                    .col(ColumnDef::new(problems::Column::Body).string().not_null())
                    .col(
                        ColumnDef::new(problems::Column::Solution)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(problems::Column::Image).string().null())
                    .primary_key(
                        Index::create()
                            .name("PK_problems")
                            .col(problems::Column::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(problems::Entity).to_owned())
            .await
    }
}
