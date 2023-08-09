/*
    Containing all the interfacing for weather's API

    Most calls look like "https://api.openweathermap.org/data/3.0/onecall?lat={lat}&lon={lon}&exclude={part}&appid={API key}"
*/

use std::{env, fmt};
use reqwest;
use serde_json;

// Struct for response for the Geo stuff
#[derive(Debug, Clone)]
pub struct GeoLocation {
    pub zip: i32,
    pub name: String,
    pub lat: f32,
    pub lon: f32,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIError {
    #[serde(rename = "code")]
    status_code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Vec<String>>,
}

impl Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display_message = String::new(self.message);
        if (self.parameters.is_some()) {
            display_message = format!("{}, Related parameters: ", display_message);
            for item in self.parameters.iter() {
                display_message = format!("{}, {}", display_message, item);
            }
        }
        write!(f, "Status: {}, Message: {}", self.status_code, display_message)
    }
}

#[derive(Debug, Clone)]
pub struct Configuration {
    base_path: String,
    user_agent: Option<String>,
    pub location: Option<GeoLocation>,
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
            Some(zip) => new_config.location = new_config.parse_zipcode(zip),
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
    pub async fn parse_zipcode(self, zipcode: &str) -> Result<GeoLocation, APIError> {
        if (!self.api_set()) {
            Err("API key not set")
        }
        if (!zipcode.contains(",")) {
            //If there is no comma in the provided zipcode, we probably forgot the ISO country code and I'm defaulting to the US
            let zipcode = zipcode + ",840";
        }

        let uri = format!("{0}geo/1.0/zip?zip={1}&appid={2}", self.base_path, zipcode, &self.api_key);
        let mut req_builder = self.client.request(reqwest::Method::GET, uri);
        req_builder = req_builder.header(reqwest::header::USER_AGENT, &self.user_agent);

        let geo_response = self.client.execute(req_builder.build()).await.ok();
        let return_status = geo_response.status();
        let return_contents = geo_response.text().await;

        //parsing the returned JSON will either get us the GeoLocation or an APIError
        serde_json::from_str(&return_contents).ok()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            base_path: "https://api.openweathermap.org/".to_owned(),
            user_agent: Some("rusty_thermostat/0.0.1".to_owned()),
            location: None,
            client: reqwest::Client::new(),
            api_key: None,
        }
    }
}