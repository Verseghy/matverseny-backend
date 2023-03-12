use entity::solutions_history::{self, constraints::*};
use entity::{problems, teams, users};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(solutions_history::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(solutions_history::Column::Id)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(solutions_history::Column::Team)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(solutions_history::Column::Problem)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(solutions_history::Column::User)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(solutions_history::Column::Solution)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(solutions_history::Column::CreatedAt)
                            .date_time()
                            .not_null()
                            .extra("DEFAULT CURRENT_TIMESTAMP".into()),
                    )
                    .primary_key(
                        Index::create()
                            .name(PK_SOLUTIONS_HISTORY)
                            .col(solutions_history::Column::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_TEAMS)
                            .from(solutions_history::Entity, solutions_history::Column::Team)
                            .to(teams::Entity, teams::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_PROBLEMS)
                            .from(
                                solutions_history::Entity,
                                solutions_history::Column::Problem,
                            )
                            .to(problems::Entity, problems::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_USERS)
                            .from(solutions_history::Entity, solutions_history::Column::User)
                            .to(users::Entity, users::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(solutions_history::Entity).to_owned())
            .await
    }
}
