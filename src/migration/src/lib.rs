pub use sea_orm_migration::prelude::*;
use async_trait::async_trait;

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Realm {
    Table,
    Id,
    Name,
    Digest,
}

#[derive(Iden)]
pub enum Player {
    Table,
    // Id,
    Hash,
    Username,
    Sid,
    Rid
}

mod m20230213_195206_create_realm_table;
mod m20230222_020006_create_player_table;
mod m20230223_212333_create_account_table;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230213_195206_create_realm_table::Migration),
            Box::new(m20230222_020006_create_player_table::Migration),
            Box::new(m20230223_212333_create_account_table::Migration),
        ]
    }
}
