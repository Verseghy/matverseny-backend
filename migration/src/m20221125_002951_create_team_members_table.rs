use entity::{team_members, teams, users};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(team_members::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(team_members::Column::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(team_members::Column::TeamId)
                            .uuid()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name("PK_team_members")
                            .col(team_members::Column::UserId)
                            .col(team_members::Column::TeamId),
                    )
                    .index(
                        Index::create()
                            .name("UC_team_members_user_id")
                            .col(team_members::Column::UserId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_team_members_user_id")
                            .from(team_members::Entity, team_members::Column::UserId)
                            .to(users::Entity, users::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_team_members_team_id")
                            .from(team_members::Entity, team_members::Column::TeamId)
                            .to(teams::Entity, teams::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(team_members::Entity).to_owned())
            .await
    }
}
