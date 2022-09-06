macro_rules! create_table_migration {
    ($entity:expr) => {
        use ::sea_orm_migration::{
            prelude::*,
            sea_orm::Schema,
        };

        #[derive(DeriveMigrationName)]
        pub struct Migration;

        #[async_trait::async_trait]
        impl MigrationTrait for Migration {
            async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager
                    .create_table(
                        Schema::new(manager.get_database_backend())
                            .create_table_from_entity($entity)
                    )
                    .await
            }

            async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager
                    .drop_table(Table::drop().table($entity).to_owned())
                    .await
            }
        }
    }
}

pub(crate) use create_table_migration;
