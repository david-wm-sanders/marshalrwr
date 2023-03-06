use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

use super::Player;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create player table
        manager.create_table(
            Table::create()
                .table(Player::Table)
                .if_not_exists()
                .col(ColumnDef::new(Player::Hash).big_integer().not_null().primary_key())
                .col(ColumnDef::new(Player::Username).string_len(32).not_null().unique_key())
                .col(ColumnDef::new(Player::Sid).big_integer().not_null())
                .col(ColumnDef::new(Player::Rid).string_len(64).not_null())
                .to_owned()
            ).await?;

        // create player username index
        manager.create_index(
            Index::create()
                .name("idx_player_username")
                .table(Player::Table)
                .col(Player::Username)
                .to_owned()
            ).await?;

        // create player sid index
        manager.create_index(
            Index::create()
                .name("idx_player_sid")
                .table(Player::Table)
                .col(Player::Sid).to_owned()          
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // drop the player sid index
        manager.drop_index(Index::drop().name("idx_player_sid").table(Player::Table).to_owned())
            .await?;

        // drop the player username index
        manager.drop_index(Index::drop().name("idx_player_username").table(Player::Table).to_owned())
            .await?;

        // drop the player table
        manager
            .drop_table(Table::drop().table(Player::Table).to_owned())
            .await?;
      
        Ok(())
    }
}
