//! # Rusty Thermostat OpenWeatherMaps API Library
//! This library holds all structs and methods to collect data from OpenWeatherMaps API

use std::{env, fmt, collections::HashMap};
use reqwest::{self, RequestBuilder};
use serde_derive::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use serde_json::Value;

// Responses from the GeoLocating API can be held here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub zip: i32,
    pub name: String,
    pub lat: f32,
    pub lon: f32,
    pub country: String,
}

impl GeoLocation {
    pub fn create_uri(&self) -> String {
        format!("lat={}&lon={}", self.lat, self.lon)
    }
}

// Responses from the Air Pollution API can be held here
#[derive(Debug, Clone, Deserialize)]
pub struct AirPollutionResponse {
    list: Vec<PollList>,
}
impl fmt::Display for AirPollutionResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "List: {:#?}", self.list)
    }
}

impl AirPollutionResponse {
    /// Consumes a AirPollutionResponse to ready it for writing to a database<br>
    /// Note: This function assumes a response with only 1 pollution check. If multiple locations were somehow returned in a single response, all but the first will be discarded
    pub fn unpack(self) -> PollUpdate {
        let current_aqi: MainAqi = self.list[0].main.clone();
        let current_pollution: Components = self.list[0].components.clone();
        PollUpdate { time: Utc::now(),
            aqi: current_aqi.aqi, co: current_pollution.co, no: current_pollution.no, no2: current_pollution.no2, o3: current_pollution.o3, so2: current_pollution.so2,
            pm2_5: current_pollution.pm2_5, pm10: current_pollution.pm10, nh3: current_pollution.nh3 }

    }
}

/// This is the structure of the write to the database <br>
/// It includes the time of the collection and all the stats collected in a flat object
#[allow(dead_code)]
pub struct PollUpdate {
    time: DateTime<Utc>,
    aqi: i8,
    co: f32,
    no: f32,
    no2: f32,
    o3: f32,
    so2: f32,
    pm2_5: f32,
    pm10: f32,
    nh3: f32,
}


/// OpenWeatherMaps uses this format to provide the pollution response. <br>
/// The response is an array but typically only has one. This structure ensures we can successfully deserialize it.
#[derive(Clone, Debug, Deserialize)]
struct PollList {
    components: Components,
    main: MainAqi,
}
impl fmt::Display for PollList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AQI: {}, Components: {}", self.main.aqi, self.components)
    }
}

/// This is the format used by OpenWeatherMaps to pass pollution amounts
#[derive(Clone, Debug, Deserialize)]
pub struct Components {
    co: f32,
    no: f32,
    no2: f32,
    o3: f32,
    so2: f32,
    pm2_5: f32,
    pm10: f32,
    nh3: f32,
}
impl fmt::Display for Components {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Carbon Monoxide: {} μg/m3, Nitrogen Monoxide: {} μg/m3, Nitrogen Dioxide: {} μg/m3, Ozone: {} μg/m3, Sulphur Dioxide: {} μg/m3, Fine Particulate Matter: {} μg/m3, Course Particulate Matter: {} μg/m3, Ammonia: {} μg/m3",
        self.co, self.no, self.no2, self.o3, self.so2, self.pm2_5, self.pm10, self.nh3)
    }
}

/// OpenWeatherMaps uses this format to pass the Air Quality Index
#[derive(Clone, Debug, Deserialize)]
pub struct MainAqi {
    aqi: i8,
}
impl fmt::Display for MainAqi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Air Quality: {}", self.aqi)
    }
}

// Response from the current weather API can be held here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherResponse {
    pub coords: (f32, f32),
    pub weather: WeatherInfo,
    pub base: String,
    #[serde(rename = "main")]
    pub temperature: TemperatureInfo,
    pub visibility: i32,
    pub wind: WindInfo,
    #[serde(skip_serializing_if="HashMap::is_empty", flatten)]
    pub rain: HashMap<String, Value>,
    #[serde(skip_serializing_if="HashMap::is_empty", flatten)]
    pub snow: HashMap<String, Value>,
    #[serde(skip_serializing_if="HashMap::is_empty", flatten)]
    pub clouds: HashMap<String, Value>,
    pub dt: i32,
    pub sys_info: SysInfo,
    pub timezone: i32,
    pub id: i32,
    pub name: String,
    pub cod: i32,
}

// Current weather stats from the WeatherResponse are stored here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherInfo {
    pub id: i32,
    pub main: String,
    pub description: String,
    pub icon: String,
}

// Current wind information from the WeatherResponse is stored here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindInfo {
    pub speed: f32,
    pub deg: i32,
    pub gust: f32,
}

// Current temperature information from the WeatherReponse is stored here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureInfo {
    pub temp: f32,
    pub feels_like: f32,
    pub pressure: i32,
    pub humidity: i32,
    pub sea_level: i32,
    pub grnd_level: i32,
}

// System information from the API and sunrise/sunset timing is stored here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysInfo {
    #[serde(rename = "type")]
    pub system_type: i32,
    pub id: i32,
    pub country: String,
    pub sunrise: i32,
    pub sunset: i32,
}

/// APIError is for containing any errors passed by the OpenWeather API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIError {
    #[serde(rename = "code")]
    status_code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Vec<String>>,
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display_message = self.message.clone();
        if self.parameters.is_some() {
            display_message = format!("{}, Related parameters: ", display_message);
            let items = self.parameters.clone().unwrap();
            for item in items.iter() {
                display_message = format!("{}, {}", display_message, item);
            }
        }
        write!(f, "Status: {}, Message: {}", self.status_code, display_message)
    }
}

// Relevant information for building the URL and containing the reqwest client are stored here
#[derive(Debug, Clone)]
pub struct Configuration {
    base_path: String,
    user_agent: Option<String>,
    location: Option<GeoLocation>,
    client: reqwest::Client,
    api_key: Option<String>,
    units: String,
}

impl Configuration {
    pub fn new() -> Configuration {
        Configuration::default()
    }
    // Build a configuration file based on environmental variables present
    // # Errors
    // When this parses a zip code into a GeoLocation, the API can return an error for a bad zipcode which will interrupt the process
    pub async fn new_env() -> Configuration {
        let mut new_config = Configuration::default();
        match env::var("RUSTY_WEATHER_BASE_PATH") {
            Ok(bpath) => new_config.base_path = bpath,
            _ => (), 
        }
        match env::var("RUSTY_WEATHER_USER_AGENT") {
            Ok(useragent) => new_config.user_agent = Some(useragent),
            _ => (),
        }
        match env::var("RUSTY_WEATHER_API_KEY") {
            Ok(apikey) => new_config.api_key = Some(apikey),
            _ => (),
        }
        match env::var("RUSTY_WEATHER_UNITS") {
            Ok(set_units) => new_config.set_units(&set_units),
            _ => (),
        }
        match env::var("RUSTY_WEATHER_LOCATION") {
            Ok(zip) => new_config.location = new_config.parse_zipcode(&zip).await.ok(),
            _ => (),
        }
        new_config
    }
    pub fn api_set(&self) -> bool {
        if self.api_key.is_some() {
            true
        } else {
            false
        }
    }
    pub fn set_units(&mut self, set_units: &str) -> () {
        let valid_units = vec!["standard".to_string(), "imperial".to_string(), "metric".to_string()];
        let lower_case_units = set_units.to_lowercase();
        if valid_units.contains(&lower_case_units) {
            self.units = lower_case_units;
        } else {
            self.units = "imperial".to_string();
        }
    }
    pub async fn parse_zipcode(&self, zipcode: &str) -> Result<GeoLocation, APIError> {
        let mut zip = zipcode.to_string();
        if !zip.contains(",") {
            //If there is no comma in the provided zipcode, we probably forgot the ISO country code and I'm defaulting to the US
            zip = format!("{},840", zipcode);
        }

        let uri = format!("geo/1.0/zip?zip={}", zip);
        let req_builder = self.build_request(&uri, reqwest::Method::GET);

        let built_req = match req_builder.build() {
            Ok(request) => request,
            Err(error) => return Err(APIError{status_code: "400".to_string(), message: error.to_string(), parameters: None})
        };

        let geo_response = match self.client.execute(built_req).await {
            Ok(resp) => resp,
            Err(error) => return Err(APIError {status_code: error.status().unwrap().to_string(), message: error.to_string(), parameters: None})
        };

        let return_contents = geo_response.text().await.ok().unwrap();

        //parsing the returned JSON will either get us the GeoLocation or an APIError
        serde_json::from_str(&return_contents).ok().unwrap()
    }
    // Accepts a URI and a reqwest method to create the RequestBuilder using the object's already established client
    pub fn build_request(&self, uri: &str, method: reqwest::Method) -> RequestBuilder {
        let mut total_url: String = format!("{0}{1}", self.base_path, uri);
        if let Some(local_api_key) = &self.api_key {
            total_url = format!("{0}&appid={1}", total_url, local_api_key);
        };
        let mut req_builder: RequestBuilder = self.client.request(method, total_url);
        if let Some(local_user_agent) = &self.user_agent {
            req_builder = req_builder.header(reqwest::header::USER_AGENT, local_user_agent);
        };
        req_builder
    }
    // Takes a complete RequestBuilder and submits it to the API
    // # Errors
    // This will return errors from the API if something doesn't work. Library will hijack a 400 error to display an internal error to building the request
    pub async fn execute_request(&self, final_request: RequestBuilder) -> Result<String, APIError> {
        let built_req = match final_request.build() {
            Ok(request) => request,
            Err(error) => return Err(APIError{status_code: "400".to_string(), message: error.to_string(), parameters: None})
        };
        let web_response = match self.client.execute(built_req).await {
            Ok(resp) => resp,
            Err(error) => return Err(APIError {status_code: error.status().unwrap().to_string(), message: error.to_string(), parameters: None})
        };
    
        let web_status = web_response.status();
        let web_content = match web_response.text().await {
            Ok(resp) => resp,
            Err(error) => return Err(APIError {status_code: web_status.to_string(), message: error.to_string(), parameters: None})
        };
        if !web_status.is_client_error() && !web_status.is_server_error() {
            Ok(web_content)
        } else {
            let api_error: APIError = serde_json::from_str(&web_content).ok().unwrap();
            Err(api_error)
        }
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
            units: "imperial".to_string(),
        }
    }
}

// Using information contained in the provided configuration, a request for current weather conditions will be processed
// # Errors
// This will return errors from the API if something doesn't work. Library will hijack a 400 error to display an internal error to building the request
pub async fn fetch_current_weather(local_config: &Configuration) -> Result<WeatherResponse, APIError> {
    let mut request_uri: String = "data/2.5/weather?".to_string();
    if let Some(local_location) = &local_config.location {
        request_uri = format!("{}{}", request_uri, local_location.create_uri());
    };
    request_uri = format!("{}&units={}", request_uri, local_config.units);
    let weather_request = local_config.build_request(&request_uri, reqwest::Method::GET);

    match local_config.execute_request(weather_request).await {
        Ok(web_response) => return serde_json::from_str(&web_response).ok().unwrap(),
        Err(error) => return Err(error),
    }
}

// Using information contained in the provided configuration, a request for current air pollution stats will be processed
// # Errors
// This will return errors from the API if something doesn't work. Library will hijack a 400 error to display an internal error to building the request
pub async fn fetch_current_air_poll(local_config: &Configuration) -> Result<AirPollutionResponse, APIError> {
    let mut request_uri: String = "data/2.5/air_pollution?".to_string();
    if let Some(local_location) = &local_config.location {
        request_uri = format!("{}{}", request_uri, local_location.create_uri());
    };
    let air_request = local_config.build_request(&request_uri, reqwest::Method::GET);

    match local_config.execute_request(air_request).await {
        Ok(web_response) => return serde_json::from_str(&web_response).ok().unwrap(),
        Err(error) => return Err(error),
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_default_basepath_not_empty() {
        let new_config = Configuration::new();

        assert!(!new_config.base_path.is_empty());
    }

    #[test]
    fn configuration_default_useragent_is_some() {
        let new_config = Configuration::new();

        assert!(new_config.user_agent.is_some());
    }

    #[test]
    fn configuration_default_location_is_none() {
        let new_config = Configuration::new();

        assert!(new_config.location.is_none());
    }

    #[test]
    fn configuration_default_client_exists() {
        let new_config = Configuration::new();

        let client_debug: String = format!("{:?}", new_config.client);

        assert!(!client_debug.is_empty());
    }

    #[test]
    fn configuration_default_api_key_is_none() {
        let new_config = Configuration::new();

        assert!(new_config.api_key.is_none());
    }

    #[test]
    fn configuration_default_units_is_imperial() {
        let new_config = Configuration::new();

        assert_eq!(new_config.units, "imperial".to_string());
    }

    #[test]
    fn configuration_default_api_key_is_set_returns_false() {
        let new_config = Configuration::new();

        assert!(!new_config.api_set());
    }

    #[test]
    fn configuration_not_default_api_key_is_set_returns_true() {
        let mut new_config = Configuration::new();

        new_config.api_key = Some("testkey".to_string());

        assert!(new_config.api_set());
    }

    #[test]
    fn configuration_set_units_to_metric() {
        let mut new_config = Configuration::new();

        new_config.set_units("metric");

        assert_eq!(new_config.units, "metric".to_string());
    }

    #[test]
    fn configuration_set_units_to_standard() {
        let mut new_config = Configuration::new();

        new_config.set_units("standard");

        assert_eq!(new_config.units, "standard".to_string());
    }

    #[test]
    fn configuration_set_units_to_imperial() {
        let mut new_config = Configuration::new();

        new_config.set_units("imperial");

        assert_eq!(new_config.units, "imperial".to_string());
    }

    #[test]
    fn configuration_set_units_to_imperial_with_nonsense() {
        let mut new_config = Configuration::new();

        new_config.set_units("absolutely hot garbage");

        assert_eq!(new_config.units, "imperial".to_string());
    }
}