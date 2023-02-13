use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Learn more at https://docs.rs/sea-query#iden
// #[derive(Iden)]
// enum Post {
//     Table,
//     Id,
//     Title,
//     Text,
// }

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Realm {
    Table,
    Id,
    Name,
    Digest,
}

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
                    .col(ColumnDef::new(Realm::Name).string_len(32).not_null())
                    .col(ColumnDef::new(Realm::Digest).string_len(64).not_null())
                    .to_owned(),
            )
            .await
            
        // todo: create index and foreign key?!
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Realm::Table).to_owned())
            .await
    }
}
