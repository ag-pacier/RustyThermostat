//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "WeatherReading")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub timestamp: DateTime,
    #[sea_orm(column_type = "Text")]
    pub condition: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    #[sea_orm(column_type = "Text")]
    pub icon: String,
    #[sea_orm(column_name = "tempReal", column_type = "Double")]
    pub temp_real: f64,
    #[sea_orm(column_name = "tempFeel", column_type = "Double")]
    pub temp_feel: f64,
    #[sea_orm(column_name = "pressureSea")]
    pub pressure_sea: i32,
    pub humidity: i32,
    #[sea_orm(column_name = "pressureGround")]
    pub pressure_ground: i32,
    pub visibility: i32,
    #[sea_orm(column_name = "windSpeed", column_type = "Double")]
    pub wind_speed: f64,
    #[sea_orm(column_name = "windDeg")]
    pub wind_deg: i32,
    #[sea_orm(column_name = "windGust", column_type = "Double")]
    pub wind_gust: f64,
    #[sea_orm(column_name = "rain1H", column_type = "Double", nullable)]
    pub rain1_h: Option<f64>,
    #[sea_orm(column_name = "rain3H", column_type = "Double", nullable)]
    pub rain3_h: Option<f64>,
    #[sea_orm(column_name = "snow1H", column_type = "Double", nullable)]
    pub snow1_h: Option<f64>,
    #[sea_orm(column_name = "snow3H", column_type = "Double", nullable)]
    pub snow3_h: Option<f64>,
    pub clouds: i32,
    pub dt: i32,
    pub sunrise: i32,
    pub sunset: i32,
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
