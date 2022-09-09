macro_rules! create_table_up {
    ($manager:expr, $entity:expr) => {
        $manager
            .create_table(
                ::sea_orm_migration::sea_orm::Schema::new($manager.get_database_backend())
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