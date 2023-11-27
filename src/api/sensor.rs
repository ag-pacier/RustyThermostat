/// Library for creating, managing and utilizing sensors that use HTTPS to communicate with the server
use sea_orm::ActiveValue::{Set, NotSet};
use serde_derive::{Serialize, Deserialize};
use chrono::{Utc, NaiveDateTime};
use std::io::Error;
use uuid::Uuid;
use serde_json;
use crate::schema::{sensors, sensor_reading_history};

// Structure to contain each web sensor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebSensor {
    pub id: Uuid,
    pub active: bool,
    pub name: String,
    pub token: String,
    pub associated_zone: Option<i32>,
    pub time_added: NaiveDateTime,
    pub time_updated: Option<NaiveDateTime>,
    pub com_type: i32,
    pub com_last: Option<NaiveDateTime>,
    pub current_temp_c: Option<f64>,
    pub current_temp_f: Option<f64>,
    pub current_humid: Option<i32>,
    pub presence: Option<bool>,
    pub threshold_open: Option<bool>,
}

impl WebSensor {
    pub fn set_humidity(&mut self, humidity: Option<i32>) -> () {
        self.current_humid = humidity;
        self.time_updated = Some(Utc::now().naive_utc());
    }
}

impl Default for WebSensor {
    fn default() -> Self {
        WebSensor {
            id: Uuid::nil(),
            active: false,
            name: "Uninitialized Web Sensor".to_string(),
            token: "PendingIssuance".to_string(),
            associated_zone: None,
            time_added: Utc::now().naive_utc(),
            time_updated: None,
            com_type: 1,
            com_last: None,
            current_temp_c: None,
            current_temp_f: None,
            current_humid: None,
            presence: None,
            threshold_open: None }
    }
}