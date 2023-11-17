//! # Rusty Thermostat Sensor library
//! These modules provide structure and methods to interact with the sensors available to the system
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fmt;

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

    /// Returns the token if there is one
    pub fn get_token(&self) -> Option<String> {
        self.token.clone()
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