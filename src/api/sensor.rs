/// Library for creating, managing and utilizing sensors that use HTTPS to communicate with the server
use sea_orm::ActiveValue::{Set, NotSet};
use serde_derive::{Serialize, Deserialize};
use chrono::{Utc, NaiveDateTime};
use std::io::Error;
use uuid::Uuid;
use serde_json;
use jsonwebtoken;
use crate::schema::{sensors, sensor_reading_history};

// Structure to contain JWT info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JWTClaims {
    pub iss: String,
    pub sub: String,
    pub groups: String,
    pub exp: u64,
}

impl JWTClaims {
    pub fn new(subject: Uuid) -> JWTClaims {
        let mut new_jwt: JWTClaims = JWTClaims::default();
        new_jwt.sub = subject.as_simple().to_string();
        new_jwt
    }
}

impl Default for JWTClaims {
    fn default() -> Self {
        let expiration: u64 = Utc::now()
            .checked_add_signed(chrono::Duration::seconds(900))
            .expect("Invalid timestamp")
            .timestamp()
            .try_into().unwrap_or(jsonwebtoken::get_current_timestamp());
        JWTClaims {
            iss: "Rusty Thermostat".to_string(),
            sub: "DebugSub".to_string(),
            groups: "WebSensor".to_string(),
            exp: expiration }
    }
}

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