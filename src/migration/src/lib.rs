pub use sea_orm_migration::prelude::*;
use async_trait::async_trait;

mod m20230213_195206_create_realm_table;
mod m20230222_020006_create_player_table;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230213_195206_create_realm_table::Migration),
            Box::new(m20230222_020006_create_player_table::Migration),
        ]
    }
}
