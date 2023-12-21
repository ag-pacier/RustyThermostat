//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "Controllers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(column_name = "Name", column_type = "Text", unique)]
    pub name: String,
    pub active: bool,
    #[sea_orm(column_name = "comType")]
    pub com_type: i32,
    #[sea_orm(column_name = "Primary")]
    pub primary: bool,
    #[sea_orm(column_name = "associatedZone")]
    pub associated_zone: Option<i32>,
    #[sea_orm(column_name = "Token", column_type = "Text", unique)]
    pub token: String,
    #[sea_orm(column_name = "timeAdded")]
    pub time_added: DateTime,
    #[sea_orm(column_name = "timeChanged")]
    pub time_changed: Option<DateTime>,
    #[sea_orm(column_name = "timeConnectLast")]
    pub time_connect_last: Option<DateTime>,
    pub capability: i32,
    #[sea_orm(column_name = "systemActive")]
    pub system_active: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::communication::Entity",
        from = "Column::ComType",
        to = "super::communication::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Communication,
    #[sea_orm(
        belongs_to = "super::env_capability::Entity",
        from = "Column::Capability",
        to = "super::env_capability::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    EnvCapability,
    #[sea_orm(
        belongs_to = "super::hva_cactivity::Entity",
        from = "Column::SystemActive",
        to = "super::hva_cactivity::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    HvaCactivity,
    #[sea_orm(
        belongs_to = "super::zones::Entity",
        from = "Column::AssociatedZone",
        to = "super::zones::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Zones,
}

impl Related<super::communication::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Communication.def()
    }
}

impl Related<super::env_capability::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EnvCapability.def()
    }
}

impl Related<super::hva_cactivity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HvaCactivity.def()
    }
}

impl Related<super::zones::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Zones.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
