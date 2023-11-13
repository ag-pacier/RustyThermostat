//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "PollutionReading")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub timestamp: DateTime,
    #[sea_orm(column_name = "AQI")]
    pub aqi: i32,
    #[sea_orm(column_name = "CO", column_type = "Double")]
    pub co: f64,
    #[sea_orm(column_name = "NO", column_type = "Double")]
    pub no: f64,
    #[sea_orm(column_name = "NO2", column_type = "Double")]
    pub no2: f64,
    #[sea_orm(column_name = "O3", column_type = "Double")]
    pub o3: f64,
    #[sea_orm(column_name = "SO2", column_type = "Double")]
    pub so2: f64,
    #[sea_orm(column_name = "PM2_5", column_type = "Double")]
    pub pm2_5: f64,
    #[sea_orm(column_name = "NH3", column_type = "Double")]
    pub nh3: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::manual_change_history::Entity")]
    ManualChangeHistory,
}

impl Related<super::manual_change_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ManualChangeHistory.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
