use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

use super::{Realm, Player};

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Account {
    Table,
    RealmId,
    Hash,
    GameVersion,
    SquadTag,
    // Color,
    MaxAuthorityReached,
    Authority,
    JobPoints,
    Faction,
    Name,
    // Alive, /* is this essential? */
    SoldierGroupId,
    SoldierGroupName,
    SquadSizeSetting,
    // how best to store loadout (varvaytya style?), backpack and stash?
    // todo!
    Kills,
    Deaths,
    TimePlayed,
    PlayerKills,
    Teamkills,
    LongestKillStreak,
    TargetsDestroyed,
    VehiclesDestroyed,
    SoldiersHealed,
    // TimesGotHealed,
    DistanceMoved,
    ShotsFired,
    ThrowablesThrown,
    RankProgression,
    // todo! stats monitors
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create account table
        manager.create_table(
            Table::create()
                .table(Account::Table)
                .if_not_exists()
                .col(ColumnDef::new(Account::RealmId).integer().not_null())
                .col(ColumnDef::new(Account::Hash).big_integer().not_null())
                .col(ColumnDef::new(Account::GameVersion).integer().not_null())
                .col(ColumnDef::new(Account::SquadTag).string_len(3).not_null())
                // note: add color if needed
                .col(ColumnDef::new(Account::MaxAuthorityReached).float().not_null())
                .col(ColumnDef::new(Account::Authority).float().not_null())
                .col(ColumnDef::new(Account::JobPoints).float().not_null())
                .col(ColumnDef::new(Account::Faction).integer().not_null())
                .col(ColumnDef::new(Account::Name).string_len(32).not_null())
                // note: add alive if needed
                .col(ColumnDef::new(Account::SoldierGroupId).integer().not_null())
                .col(ColumnDef::new(Account::SoldierGroupName).string_len(32).not_null())
                .col(ColumnDef::new(Account::SquadSizeSetting).integer().not_null())
                // rip squad config index
                // todo: add loadout, backpack, stash
                .col(ColumnDef::new(Account::Kills).integer().not_null())
                .col(ColumnDef::new(Account::Deaths).integer().not_null())
                .col(ColumnDef::new(Account::TimePlayed).integer().not_null())
                .col(ColumnDef::new(Account::PlayerKills).integer().not_null())
                .col(ColumnDef::new(Account::Teamkills).integer().not_null())
                .col(ColumnDef::new(Account::LongestKillStreak).integer().not_null())
                .col(ColumnDef::new(Account::TargetsDestroyed).integer().not_null())
                .col(ColumnDef::new(Account::VehiclesDestroyed).integer().not_null())
                .col(ColumnDef::new(Account::SoldiersHealed).integer().not_null())
                // add times_got_healed if ever recorded :D
                .col(ColumnDef::new(Account::DistanceMoved).float().not_null())
                .col(ColumnDef::new(Account::ShotsFired).integer().not_null())
                .col(ColumnDef::new(Account::ThrowablesThrown).integer().not_null())
                .col(ColumnDef::new(Account::RankProgression).float().not_null())
                // setup composite primary key
                .primary_key(Index::create().col(Account::RealmId).col(Account::Hash))
                // add foreign keys
                .foreign_key(ForeignKey::create().name("fk-account-realm_id")
                    .from(Account::Table, Account::RealmId)
                    .to(Realm::Table, Realm::Id))
                .foreign_key(ForeignKey::create().name("fk-account-hash")
                    .from(Account::Table, Account::Hash)
                    .to(Player::Table, Player::Hash))
                .to_owned()
            ).await?;
            
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // drop the player table
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await?;
      
        Ok(())
    }
}
