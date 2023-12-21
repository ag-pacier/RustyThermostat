//! # Rusty Thermostat OpenWeatherMaps API Library
//! This library holds all structs and methods to collect data from OpenWeatherMaps API

use std::{env, fmt};
use reqwest::{self, RequestBuilder};
use sea_orm::ActiveValue::{Set, NotSet};
use serde_derive::{Serialize, Deserialize};
use chrono::Utc;
use serde_json;
use crate::schema::{weather_reading, pollution_reading};

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
    pub fn generate_db_model(self) -> pollution_reading::ActiveModel {
        let current_aqi: MainAqi = self.list[0].main.clone();
        let current_pollution: Components = self.list[0].components.clone();
        pollution_reading::ActiveModel { timestamp: Set(Utc::now().naive_utc()),
            id: NotSet, aqi: Set(current_aqi.aqi.into()), co: Set(current_pollution.co.into()),
            no: Set(current_pollution.no.into()), no2: Set(current_pollution.no2.into()),
            o3: Set(current_pollution.o3.into()), so2: Set(current_pollution.so2.into()),
            pm2_5: Set(current_pollution.pm2_5.into()), pm10: Set(current_pollution.pm10.into()),
            nh3: Set(current_pollution.nh3.into()) }

    }
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
    // Weather conditions
    weather: WeatherInfo,
    // Internally used string I'm keeping for now. Example has "stations"
    base: String,
    // Temperature information
    #[serde(rename = "main")]
    temperature: TemperatureInfo,
    // Visibility based on conditions in selected units
    visibility: i32,
    // Wind information
    wind: WindInfo,
    // Rain accumulation
    rain: Option<RainInfo>,
    // Snow accumulation
    snow: Option<SnowInfo>,
    // Cloudiness percentage
    clouds: CloudInfo,
    // Time of calculation unix UTC
    dt: i32,
    // System information with sunrise and sunset
    sys_info: SysInfo,
}

impl WeatherResponse {
    // Get a copy of the WeatherInfo contained in a WeatherResponse
    pub fn get_conditions_info(&self) -> WeatherInfo {
        self.weather.clone()
    }
    // Get a copy of the TemperatureInfo contained in a WeatherResponse
    pub fn get_temp_info(&self) -> TemperatureInfo {
        self.temperature.clone()
    }
    // Get the current visibility in a WeatherResponse
    pub fn get_visibility(&self) -> i32 {
        self.visibility.clone()
    }
    // Get a copy of the WindInfo contained in a WeatherResponse
    pub fn get_wind_info(&self) -> WindInfo {
        self.wind.clone()
    }
    // Get a copy of the RainInfo contained in a WeatherResponse
    // possible that it will be None
    pub fn get_rain_info(&self) -> Option<RainInfo> {
        self.rain.clone()
    }
    // Get a copy of the SnowInfo contained in a WeatherResponse
    // possible that it will be None
    pub fn get_snow_info(&self) -> Option<SnowInfo> {
        self.snow.clone()
    }
    // Get the current cloudiness percentage
    pub fn get_cloudiness(&self) -> i32 {
        self.clouds.all.clone()
    }
    // Get the day's sunrise in unix UTC
    pub fn get_sunrise(&self) -> i32 {
        self.sys_info.sunrise.clone()
    }
    // Get the day's sunset in unix UTC
    pub fn get_sunset(&self) -> i32 {
        self.sys_info.sunset.clone()
    }
    // Consumes a WeatherResponse into an ActiveModel to be put into the DB
    pub fn generate_db_model(self) -> weather_reading::ActiveModel {
        let mut reading: weather_reading::ActiveModel = weather_reading::ActiveModel {
            id: NotSet,
            timestamp: Set(Utc::now().naive_utc()),
            condition: Set(self.weather.main),
            description: Set(self.weather.description),
            icon: Set(self.weather.icon),
            temp_real: Set(self.temperature.temp.into()),
            temp_feel: Set(self.temperature.feels_like.into()),
            pressure_sea: Set(self.temperature.pressure),
            humidity: Set(self.temperature.humidity),
            pressure_ground: Set(self.temperature.grnd_level),
            visibility: Set(self.visibility),
            wind_speed: Set(self.wind.speed.into()),
            wind_deg: Set(self.wind.deg),
            wind_gust: Set(self.wind.gust.into()),
            rain1_h: NotSet,
            rain3_h: NotSet,
            snow1_h: NotSet,
            snow3_h: NotSet,
            clouds: Set(self.clouds.all),
            dt: Set(self.dt),
            sunrise: Set(self.sys_info.sunrise),
            sunset: Set(self.sys_info.sunset),
        };
        let rain: Option<RainInfo> = self.rain;
        let snow: Option<SnowInfo> = self.snow;

        if rain.is_some() {
            let unpacked: RainInfo = rain.unwrap();
            reading.rain1_h = Set(Some(unpacked.onehour.into()));
            if unpacked.threehour.is_some() {
                let threehour: f32 = unpacked.threehour.unwrap();
                reading.rain3_h = Set(Some(threehour));
            }
        }
        if snow.is_some() {
            let unpacked: SnowInfo = snow.unwrap();
            reading.snow1_h = Set(Some(unpacked.onehour.into()));
            if unpacked.threehour.is_some() {
                let threehour: f32 = unpacked.threehour.unwrap();
                reading.snow3_h = Set(Some(threehour));
            }
        }
        reading
    }

    pub fn test_my_weather() -> WeatherResponse {
        let weath_info: WeatherInfo = WeatherInfo {
            id: 69,
            main: "Sunny".to_string(),
            description: "It's totally sunny".to_string(),
            icon: "2px.png".to_string() };
        let win_info: WindInfo = WindInfo {
            speed: 69.6,
            deg: 320,
            gust: 69.6 };
        let temp_info: TemperatureInfo = TemperatureInfo {
            temp: 70.0,
            feels_like: 69.0,
            pressure: 20,
            humidity: 30,
            grnd_level: 30 };
        let system_i: SysInfo = SysInfo {
            country: "USofA".to_string(),
            sunrise: 3299980,
            sunset: 3367594 };

        WeatherResponse {
            weather: weath_info,
            base: "Stations".to_string(),
            temperature: temp_info,
            visibility: 100000,
            wind: win_info,
            rain: None,
            snow: None,
            clouds: CloudInfo { all: 15 },
            dt: 23239239,
            sys_info: system_i }
    }
}

// Current weather stats from the WeatherResponse are stored here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherInfo {
    // Weather condition id
    id: i32,
    // Group of weather parameters (Rain, Snow, Clouds etc.)
    main: String,
    // Weather condition within the group
    description: String,
    // Weather icon id which allows you to pull their icon for the conditions
    icon: String,
}

impl WeatherInfo {
    // Get the URL to the icon for the current condition
    pub fn get_icon(&self) -> String {
        format!("https://openweathermap.org/img/wn/{}@2x.png", self.icon.clone())
    }
    // Get the weather group
    pub fn get_weather_head(&self) -> String {
        self.main.clone()
    }
    // Get the weather description
    pub fn get_weather_description(&self) -> String {
        self.description.clone()
    }
}

// Current wind information from the WeatherResponse is stored here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindInfo {
    // Wind speed based on units selected
    speed: f32,
    // Wind direction, degrees (meteorological)
    deg: i32,
    // Wind gusts based on units selected
    gust: f32,
}

impl WindInfo {
    // Get a copy of the wind speed in WindInfo
    pub fn get_wind_speed(&self) -> f32 {
        self.speed.clone()
    }
    // Get a copy of wind direction in WindInfo
    pub fn get_wind_direction(&self) -> i32 {
        self.deg.clone()
    }
    // Get the gust speed in WindInfo
    pub fn get_wind_gust(&self) -> f32 {
        self.gust.clone()
    }
}

// Current temperature information from the WeatherReponse is stored here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureInfo {
    // Recorded temperature
    temp: f32,
    // Temperature "feels like" given conditions
    feels_like: f32,
    // Atmospheric pressure at sea level
    pressure: i32,
    // Humidity in percentage
    humidity: i32,
    // Atmospheric pressure at ground level
    grnd_level: i32,
}

impl TemperatureInfo {
    // Get a copy of the temperature contained in TemperatureInfo
    pub fn get_temp(&self) -> f32 {
        self.temp.clone()
    }
    // Get a copy of the human feel temperature contained in TemperatureInfo
    pub fn get_feels_like(&self) -> f32 {
        self.feels_like.clone()
    }
    // Get a copy of the sea level atomospheric pressure contained in TemperatureInfo
    pub fn get_sea_level_pressure(&self) -> i32 {
        self.pressure.clone()
    }
    // Get a copy of the humidity percentage contained in TemperatureInfo
    pub fn get_humidity(&self) -> i32 {
        self.humidity.clone()
    }
    // Get a copy of the ground level atomospheric pressure contained in TemperatureInfo
    pub fn get_ground_level_pressure(&self) -> i32 {
        self.grnd_level.clone()
    }
}

// System information: sunrise/sunset timing is stored here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysInfo {
    #[serde(rename = "type")]
    // Country code from where the check came from
    country: String,
    // Timing of the sunrise in unix UTC
    sunrise: i32,
    // Timing of the sunset in unix UTC
    sunset: i32,
}

// Rain information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RainInfo {
    // Rain accumulation in 1 hour
    #[serde(rename = "1h")]
    onehour: f32,
    // Rain accumulation in 3 hours
    #[serde(rename = "3h")]
    threehour: Option<f32>,
}

// Snow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnowInfo {
    // Snow accumulation in 1 hour
    #[serde(rename = "1h")]
    onehour: f32,
    // Snow accumulation in 3 hours
    #[serde(rename = "3h")]
    threehour: Option<f32>,
}

// Cloudiness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudInfo {
    // Percentage of cloudiness
    all: i32,
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
#[derive(Debug, Clone, Deserialize)]
pub struct Configuration {
    pub base_path: String,
    pub user_agent: Option<String>,
    pub location: Option<GeoLocation>,
    #[serde(skip_deserializing)]
    pub client: reqwest::Client,
    pub api_key: Option<String>,
    pub units: String,
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