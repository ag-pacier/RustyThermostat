//! Sea ORM models for all database related activities
//! using: https://www.sea-ql.org/SeaORM/docs/generate-entity/entity-structure/
//! using: https://stackoverflow.com/questions/7296846/how-to-implement-one-to-one-one-to-many-and-many-to-many-relationships-while-de#:~:text=One-to-many%3A%20Use%20a%20foreign%20key%20on%20the%20many,%22many%22%20side%20Many-to-many%3A%20Use%20a%20junction%20table%20%28example%29%3A
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "CurrentConditions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pollution: i32,
    pub weather: i32,
    pub home: i32,
    pub trippedalerts: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
}

//impl Related<super::fruit::Entity> for Entity {
//}

impl ActiveModelBehavior for ActiveModel {}