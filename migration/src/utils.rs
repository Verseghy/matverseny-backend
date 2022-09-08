macro_rules! create_table_migration {
    ($entity:expr) => {
        use ::sea_orm_migration::{prelude::*, sea_orm::Schema};

        #[derive(DeriveMigrationName)]
        pub struct Migration;

        #[async_trait::async_trait]
        impl MigrationTrait for Migration {
            async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager
                    .create_table(
                        Schema::new(manager.get_database_backend())
                            .create_table_from_entity($entity)
                            .if_not_exists()
                            .to_owned(),
                    )
                    .await
            }

            async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager
                    .drop_table(Table::drop().if_exists().table($entity).to_owned())
                    .await
            }
        }
    };
}

pub(crate) use create_table_migration;

macro_rules! create_table_up {
    ($manager:expr, $entity:expr) => {
        $manager
            .create_table(
                Schema::new($manager.get_database_backend())
                    .create_table_from_entity($entity)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;
    };
}

pub(crate) use create_table_up;

macro_rules! create_table_down {
    ($manager:expr, $entity:expr) => {
        $manager
            .drop_table(Table::drop().if_exists().table($entity).to_owned())
            .await?;
    };
}

pub(crate) use create_table_down;
