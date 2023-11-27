//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "Alerts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(column_name = "Name", column_type = "Text", unique)]
    pub name: String,
    pub active: bool,
    pub tripped: bool,
    #[sea_orm(column_name = "comType")]
    pub com_type: Option<i32>,
    #[sea_orm(column_name = "associatedSchedule")]
    pub associated_schedule: Option<i32>,
    #[sea_orm(column_name = "associatedZone")]
    pub associated_zone: Option<i32>,
    #[sea_orm(column_name = "Actions", column_type = "Text", nullable)]
    pub actions: Option<String>,
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
        belongs_to = "super::schedules::Entity",
        from = "Column::AssociatedSchedule",
        to = "super::schedules::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Schedules,
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

impl Related<super::schedules::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Schedules.def()
    }
}

impl Related<super::zones::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Zones.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
