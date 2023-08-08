/*
    Containing all the interfacing for weather's API

    Most calls look like "https://api.openweathermap.org/data/3.0/onecall?lat={lat}&lon={lon}&exclude={part}&appid={API key}"
*/

use std::env;
use reqwest;

#[derive(Debug, Clone)]
pub struct Configuration {
    base_path: String,
    user_agent: Option<String>,
    pub location: Option<(f32, f32)>,
    client: reqwest::Client,
    api_key: Option<String>,
}

impl Configuration {
    pub fn new() -> Configuration {
        Configuration::default()
    }
    pub fn new_env() -> Configuration {
        let mut new_config = Configuration::default();
        match env::var("WEATHER_BASE_PATH") {
            Some(bpath) => new_config.base_path = bpath,
            _ => (), 
        }
        match env::var("WEATHER_USER_AGENT") {
            Some(useragent) => new_config.user_agent = useragent,
            _ => (),
        }
        match env::var("WEATHER_LOCATION") {
            Some(todo) => new_config.location = todo,
            _ => (),
        }
        match env::var("WEATHER_API_KEY") {
            Some(apikey) => new_config.api_key = apikey,
            _ => (),
        }
        new_config
    }
    pub fn api_set(self) -> bool {
        match self.api_key {
            Some => true,
            _ => false,
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            base_path: "https://api.openweathermap.org/data/3.0/onecall?".to_owned(),
            user_agent: Some("rusty_thermostat/0.0.1".to_owned()),
            location: None,
            client: reqwest::Client::new(),
            api_key: None,
        }
    }
}