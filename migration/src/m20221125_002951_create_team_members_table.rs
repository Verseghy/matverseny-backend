use entity::{
    team_members::{self, constraints::*},
    teams, users,
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
                            .name(PK_TEAM_MEMBERS)
                            .col(team_members::Column::UserId)
                            .col(team_members::Column::TeamId),
                    )
                    .index(
                        Index::create()
                            .name(UC_TEAM_MEMBERS_USER_ID)
                            .col(team_members::Column::UserId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_TEAM_MEMBERS_USER_ID)
                            .from(team_members::Entity, team_members::Column::UserId)
                            .to(users::Entity, users::Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_TEAM_MEMBERS_TEAM_ID)
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
