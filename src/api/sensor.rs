/// Library for creating, managing and utilizing sensors that use HTTPS to communicate with the server
use sea_orm::ActiveValue::{Set, NotSet};
use serde_derive::{Serialize, Deserialize};
use chrono::{Utc, NaiveDateTime};
use uuid::Uuid;
use serde_json;
use rand::{thread_rng, Rng};
use crate::schema::{sensors, sensor_reading_history};

// Structure to contain each web sensor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebSensor {
    pub id: Uuid,
    pub active: bool,
    pub name: String,
    pub token: [u8; 32],
    pub associated_zone: Option<i32>,
    pub time_added: NaiveDateTime,
    pub time_updated: Option<NaiveDateTime>,
    pub com_type: i32,
    pub com_last: Option<NaiveDateTime>,
    pub current_temp_c: Option<f32>,
    pub current_temp_f: Option<f32>,
    pub current_humid: Option<i32>,
    pub presence: Option<bool>,
    pub threshold_open: Option<bool>,
}

impl WebSensor {
    pub fn new_sensor(name: String, zone: Option<i32>) -> WebSensor {
        let mut new_sensor: WebSensor = WebSensor::default();
        new_sensor.set_name(name);
        new_sensor.set_zone(zone);
        new_sensor.id = Uuid::new_v4();
        new_sensor.set_token();
        new_sensor
    }
    fn update_time(&mut self) -> () {
        self.time_updated = Some(Utc::now().naive_utc());
    }
    fn update_com_last(&mut self) -> () {
        self.com_last = Some(Utc::now().naive_utc());
    }
    pub fn set_token(&mut self) -> () {
        let mut rng: rand::prelude::ThreadRng = thread_rng();
        let mut rand_range: [u8; 32] = [0; 32];
        let mut i: usize = 0;
        while i < rand_range.len() {
            rand_range[i] = rng.gen();
            i = i + 1;
        }
        self.token = rand_range;
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
    pub fn set_f_temp(&mut self, f_temp: Option<f32>) -> () {
        self.current_temp_f = f_temp;
        self.update_time();
    }
    pub fn set_c_temp(&mut self, c_temp: Option<f32>) -> () {
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
            token: [0; 32],
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

// Structure to contain each web sensor reading
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebSensorReading {
    pub id: i32,
    pub sensor_id: Uuid,
    pub associated_zone: Option<i32>,
    pub timestamp: NaiveDateTime,
    pub current_temp_c: Option<f32>,
    pub current_temp_f: Option<f32>,
    pub current_humid: Option<i32>,
    pub presence: Option<bool>,
    pub threshold_open: Option<bool>,
}