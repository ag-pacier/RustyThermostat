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
    pub fn new_sensor(name: String, zone: Option<i32>) -> WebSensor {
        let mut new_sensor: WebSensor = WebSensor::default();
        new_sensor.set_name(name);
        new_sensor.set_zone(zone);
        //TODO: UUID check and input into DB
        new_sensor
    }
    fn update_time(&mut self) -> () {
        self.time_updated = Some(Utc::now().naive_utc());
    }
    fn update_com_last(&mut self) -> () {
        self.com_last = Some(Utc::now().naive_utc());
    }
    pub fn set_humidity(&mut self, humidity: Option<i32>) -> () {
        self.current_humid = humidity;
        self.update_time();
    }
    pub fn set_name(&mut self, new_name: String) -> () {
        self.name = new_name;
        self.update_time();
    }
    pub fn set_active(&mut self, is_active: bool) -> () {
        self.active = is_active;
        self.update_time();
    }
    pub fn set_pres(&mut self, is_pres: Option<bool>) -> () {
        self.presence = is_pres;
        self.update_time();
    }
    pub fn set_thresh(&mut self, is_thresh: Option<bool>) -> () {
        self.threshold_open = is_thresh;
        self.update_time();
    }
    pub fn set_zone(&mut self, new_zone: Option<i32>) -> () {
        self.associated_zone = new_zone;
        self.update_time();
    }
    pub fn set_f_temp(&mut self, f_temp: Option<f64>) -> () {
        self.current_temp_f = f_temp;
        self.update_time();
    }
    pub fn set_c_temp(&mut self, c_temp: Option<f64>) -> () {
        self.current_temp_c = c_temp;
        self.update_time();
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