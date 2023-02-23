use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

use super::Realm;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create the realm table
        manager
            .create_table(
                Table::create()
                    .table(Realm::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Realm::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Realm::Name).string_len(32).not_null().unique_key())
                    .col(ColumnDef::new(Realm::Digest).string_len(64).not_null())
                    .to_owned(),
            )
            .await?;
            
        // todo: create index and foreign key?!
        manager.create_index(Index::create().name("idx_realm_name").table(Realm::Table).col(Realm::Name).to_owned()).await?;
        
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // drop the realm name index
        manager.drop_index(Index::drop().name("idx_realm_name").table(Realm::Table).to_owned()).await?;
        
        // drop the realm table
        manager
            .drop_table(Table::drop().table(Realm::Table).to_owned())
            .await?;
            
        Ok(())
    }
}
