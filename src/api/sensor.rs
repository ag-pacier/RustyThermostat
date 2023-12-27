/// Library for creating, managing and utilizing sensors that use HTTPS to communicate with the server
use sea_orm::ActiveValue::{Set, NotSet};
use serde_derive::{Serialize, Deserialize};
use chrono::{Utc, NaiveDateTime};
use uuid::Uuid;
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

    pub fn generate_db_model_new(&self) -> sensors::ActiveModel {
        sensors::ActiveModel {
            id: Set(self.id),
            active: Set(self.active),
            name: Set(self.name.clone()),
            token: Set(self.token.into()),
            associated_zone: Set(self.associated_zone),
            time_added: Set(self.time_added),
            time_updated: Set(self.time_updated),
            com_type: Set(self.com_type),
            com_last: Set(self.com_last),
            current_temp_f: Set(self.current_temp_f),
            current_temp_c: Set(self.current_temp_c),
            current_humid: Set(self.current_humid),
            presence: Set(self.presence),
            threshold_open: Set(self.threshold_open) }
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

impl WebSensorReading {
    pub fn blank_new() -> WebSensorReading {
        WebSensorReading::default()
    }

    pub fn new(source: Uuid) -> WebSensorReading {
        let mut new_reading: WebSensorReading = WebSensorReading::default();
        new_reading.set_uuid(source);
        new_reading
    }

    pub fn from_web_sensor(sensor: &WebSensor) -> WebSensorReading {
        let local_sensor: WebSensor = sensor.clone();
        let mut reading: WebSensorReading = WebSensorReading::default();
        reading.set_uuid(local_sensor.id);
        if let Some(zone) = local_sensor.associated_zone {
            reading.associated_zone = Some(zone);
        }
        if let Some(temp) = local_sensor.current_temp_c {
            reading.current_temp_c = Some(temp);
        }
        if let Some(temp) = local_sensor.current_temp_f {
            reading.current_temp_f = Some(temp);
        }
        if let Some(humid) = local_sensor.current_humid {
            reading.current_humid = Some(humid);
        }
        if let Some(presence) = local_sensor.presence {
            reading.presence = Some(presence);
        }
        if let Some(threshold) = local_sensor.threshold_open {
            reading.threshold_open = Some(threshold);
        }
        reading
    }

    pub fn set_uuid(&mut self, source: Uuid) -> () {
        self.sensor_id = source;
    }

    pub fn generate_db_model(&self) -> sensor_reading_history::ActiveModel {
        sensor_reading_history::ActiveModel {
            id: NotSet,
            sensor_id: Set(self.sensor_id),
            timestamp: Set(self.timestamp),
            reading_temp_f: Set(self.current_temp_f),
            reading_temp_c: Set(self.current_temp_c),
            reading_humidity: Set(self.current_humid),
            reading_presence: Set(self.presence),
            reading_threshold_open: Set(self.threshold_open) }
    }
}

impl Default for WebSensorReading {
    fn default() -> Self {
        WebSensorReading {
            id: 0,
            sensor_id: Uuid::nil(),
            associated_zone: None,
            timestamp: Utc::now().naive_utc(),
            current_temp_c: None,
            current_temp_f: None,
            current_humid: None,
            presence: None,
            threshold_open: None }
    }
}

pub fn new_reading(sensor: &WebSensor) -> Result<sensor_reading_history::ActiveModel, sea_orm::error::RuntimeErr> {
    let mut reading: WebSensorReading = WebSensorReading::from_web_sensor(sensor);
    reading.timestamp = Utc::now().naive_utc();
    let reading_model: sensor_reading_history::ActiveModel = reading.generate_db_model();
    Ok(reading_model)
}

pub fn new_sensor(sensor_name: &str, sens_zone: Option<i32>) -> Result<WebSensor, sea_orm::error::RuntimeErr> {
    let mut new_sens: WebSensor = WebSensor::new_sensor(sensor_name.to_string(), sens_zone);
    new_sens.time_added = Utc::now().naive_utc();
    Ok(new_sens)
}