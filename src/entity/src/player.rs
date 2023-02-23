//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "player")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub hash: i32,
    pub username: String,
    pub sid: i32,
    pub rid: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::realm::Entity> for Entity {
    fn to() -> RelationDef {
        super::account::Relation::Realm.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::account::Relation::Player.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
