//! # Rusty Thermostat Sensor library
//! These modules provide structure and methods to interact with the sensors available to the system

use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fmt;
use sea_orm::ActiveValue::{Set, NotSet};
use crate::schema::{sensors, sensor_reading_history};

pub mod pi_ser;

// Note to self, be sure to put the database logic for this stuff on this level
// That way it's "centralized" and each weird communication type can just rely on this
// TODO: add logic for dealing with sensor reading history table

/// Sensor struct that holds all its relevant information
#[derive(Clone, Debug)]
pub struct Sensor{
    pub id: Uuid,
    pub active: bool,
    pub name: String,
    token: Option<String>,
    pub associated_zone: Option<i32>,
    pub time_added: DateTime<Utc>,
    pub time_updated: Option<DateTime<Utc>>,
    pub com_type: i32,
    pub last_reading: Option<SensorReading>,
}

impl Sensor {
    /// Creating a new sensor gives it a fresh UUID v4, a placeholder name of "New Sensor" and
    /// adds the timestamp when this was ran in UTC
    pub fn new() -> Sensor {
        Sensor::default()
    }

    /// Returns the token. Will be None if nothing is set
    pub fn get_token(&self) -> String {
        match self.token.clone() {
            Some(toke) => toke,
            None => "N/A".to_string(),
        }
    }

    /// Consumes a sensor to create an ActiveModel to put in the db
    pub fn generate_db_model_new_sensor(self) -> sensors::ActiveModel {
        let token: String = self.get_token();
        sensors::ActiveModel {
            id: Set(self.id),
            active: Set(self.active),
            name: Set(self.name),
            token: Set(token),
            associated_zone: Set(self.associated_zone),
            time_added: Set(Utc::now().naive_utc()),
            time_updated: NotSet,
            com_type: Set(self.com_type),
            com_last: NotSet,
            current_temp: NotSet,
            current_humid: NotSet,
            presence: NotSet,
            threshold_open: NotSet,
        }
    }
}

impl Default for Sensor {
    fn default() -> Self {
        Sensor {
            id: Uuid::new_v4(),
            active: false,
            name: "New Sensor".to_string(),
            token: None,
            associated_zone: None,
            time_added: Utc::now(),
            time_updated: None,
            com_type: 0,
            last_reading: None
        }
    }
}

/// Struct for holding sensor readings
#[derive(Clone, Debug)]
pub struct SensorReading {
    device_id: Uuid,
    timestamp: DateTime<Utc>,
    temp_c_reading: Option<f32>,
    temp_f_reading: Option<f32>,
    humid_reading: Option<i32>,
    presence: Option<bool>,
    threshold_open: Option<bool>,
}

impl SensorReading {
    /// Creating a new sensor reading struct only needs a string slice of the ID that will be doing the reading
    /// Helper functions are used after the fact to add the reading
    /// # Errors
    /// If the provided string slice can't be parsed into a UUID, this will result in failure
    pub fn new(id: &str) -> Result<SensorReading, uuid::Error> {
        let reading: SensorReading = SensorReading {
            device_id: Uuid::parse_str(id)?,
            timestamp: Utc::now(),
            temp_c_reading: None,
            temp_f_reading: None,
            humid_reading: None,
            presence: None,
            threshold_open: None };
        Ok(reading)
    }

    /// Add the temperature reading in Celsius to the sensor reading
    pub fn append_temp_c(mut self, temp: f32) -> () {
        self.temp_c_reading = Some(temp);
    }

    /// Add the temperature reading in Fahrenheit to the sensor reading
    pub fn append_temp_f(mut self, temp: f32) -> () {
        self.temp_f_reading = Some(temp);
    }

    /// Add the humidity percentage to the sensor reading
    pub fn append_humid(mut self, humid: i32) -> () {
        self.humid_reading = Some(humid);
    }

    /// Add the presence status to the sensor reading
    pub fn append_presence(mut self, pres: bool) -> () {
        self.presence = Some(pres);
    }

    /// Add the threshold status to the sensor reading
    pub fn append_threshold(mut self, thresh: bool) -> () {
        self.threshold_open = Some(thresh);
    }

    /// Convert Celsius from f32 to f64
    pub fn convert_cel(&self) -> Option<f64> {
        match self.temp_c_reading {
            Some(temp_c) => Some(temp_c.into()),
            None => None
        }
    }

    /// Convert Fahrenheit from f32 to f64
    pub fn convert_fah(&self) -> Option<f64> {
        match self.temp_f_reading {
            Some(temp_f) => Some(temp_f.into()),
            None => None
        }
    }

    /// Consume a sensor reading to put in the sensor reading history table
    pub fn generate_db_model(self) -> sensor_reading_history::ActiveModel {
        sensor_reading_history::ActiveModel {
            id: NotSet,
            sensor_id: Set(self.device_id),
            timestamp: Set(Utc::now().naive_utc()),
            reading_temp_c: Set(self.convert_cel()),
            reading_temp_f: Set(self.convert_fah()),
            reading_humidity: Set(self.humid_reading),
            reading_presence: Set(self.presence),
            reading_threshold_open: Set(self.threshold_open)
        }
    }
}

impl fmt::Display for SensorReading {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display_message: String = format!("Source: {}, Timestamp: {}", self.device_id.to_string(), self.timestamp.to_string());
        if let Some(temp) = self.temp_c_reading {
            display_message = format!("{}, Celsius: {}", display_message, temp);
        }
        if let Some(temp) = self.temp_f_reading {
            display_message = format!("{}, Fahrenheit: {}", display_message, temp);
        }
        if let Some(humidity) = self.humid_reading {
            display_message = format!("{}, Humidity: {}", display_message, humidity);
        }
        if let Some(presence) = self.presence {
            display_message = format!("{}, Presence: {}", display_message, presence);
        }
        if let Some(thresher) = self.threshold_open {
            display_message = format!("{}, Threshold Open: {}", display_message, thresher);
        }
        write!(f, "{}", display_message)
    }
}