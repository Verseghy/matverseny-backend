use std::time::UNIX_EPOCH;

use entity::times::{self, constrains::*};
use sea_orm_migration::{
    prelude::*,
    sea_orm::{EntityTrait, Set, prelude::DateTimeUtc},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(times::Entity)
                    .if_not_exists()
                    .col(ColumnDef::new(times::Column::Name).string().not_null())
                    .col(
                        ColumnDef::new(times::Column::Time)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(PK_TIMES)
                            .col(times::Column::Name)
                            .primary(),
                    )
                    .to_owned(),
            )
            .await?;

        let start_time = times::ActiveModel {
            name: Set("start_time".to_owned()),
            time: Set(DateTimeUtc::from(UNIX_EPOCH)),
        };

        let end_time = times::ActiveModel {
            name: Set("end_time".to_owned()),
            time: Set(DateTimeUtc::from(UNIX_EPOCH)),
        };

        times::Entity::insert_many([start_time, end_time])
            .exec(manager.get_connection())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(times::Entity).to_owned())
            .await
    }
}
