use entity::problems_order;
use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // manager
        //     .create_table(
        //         Table::create()
        //             .table(problems_order::Entity)
        //             .if_not_exists()
        //             .col(ColumnDef::new(problems_order::Column::Id).uuid().not_null())
        //             .col(ColumnDef::new(problems_order::Column::Next).uuid().null())
        //             .index(
        //                 Index::create()
        //                     .name("PK_problems_order")
        //                     .col(problems_order::Column::Id)
        //                     .primary()
        //                     .deferrable(Deferrable::DeferrableInitiallyImmediate),
        //             )
        //             .index(
        //                 Index::create()
        //                     .name("UC_problems_order_next")
        //                     .col(problems_order::Column::Next)
        //                     .unique(), // TODO: in next sea-orm update use nulls_not_distinct()
        //             )
        //             .foreign_key(
        //                 ForeignKey::create()
        //                     .name("FK_problems_order_id")
        //                     .from(problems_order::Entity, problems_order::Column::Id)
        //                     .to(problems::Entity, problems::Column::Id),
        //             )
        //             .foreign_key(
        //                 ForeignKey::create()
        //                     .name("FK_problems_order_next")
        //                     .from(problems_order::Entity, problems_order::Column::Next)
        //                     .to(problems_order::Entity, problems_order::Column::Id)
        //                     .deferrable(Deferrable::DeferrableInitiallyImmediate),
        //             )
        //             .to_owned();
        //     )
        //     .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                r#"CREATE TABLE IF NOT EXISTS "problems_order" (
                    "id" uuid NOT NULL,
                    "next" uuid NULL,

                    CONSTRAINT "PK_problems_order"
                        PRIMARY KEY ("id"),
                    CONSTRAINT "UC_problems_order_next"
                        UNIQUE NULLS NOT DISTINCT ("next")
                        DEFERRABLE INITIALLY IMMEDIATE,
                    CONSTRAINT "FK_problems_order_id"
                        FOREIGN KEY ("id")
                        REFERENCES "problems" ("id")
                        DEFERRABLE INITIALLY IMMEDIATE,
                    CONSTRAINT "FK_problems_order_next"
                        FOREIGN KEY ("next")
                        REFERENCES "problems_order" ("id")
                        DEFERRABLE INITIALLY IMMEDIATE
                )"#
                .to_owned(),
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(problems_order::Entity).to_owned())
            .await
    }
}
