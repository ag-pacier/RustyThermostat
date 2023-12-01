use figment::{Figment, providers::{Format, Toml, Env}};
use serde_derive::Deserialize;

pub mod weather;
pub mod schema;
pub mod dbman;

#[macro_use] extern crate rocket;

#[derive(Clone, Debug, Deserialize)]
struct AppConfiguration {
    weather: WeatherSettings,
    database: DatabaseSettings,
    logging: LogSettings,
}

#[derive(Clone, Debug, Deserialize)]
struct LogSettings {
    enabled: bool,
    log_level: Option<String>,
    log_location: Option<String>
}

#[derive(Clone, Debug, Deserialize)]
struct WeatherSettings {
    monitor_weather: bool,
    monitor_pollution: bool,
    zip_code: String,
    country: Option<String>,
    units: Option<String>,
    openweather_apikey: Option<String>
}

#[derive(Clone, Debug, Deserialize)]
struct DatabaseSettings {
    database_type: String,
    host: String,
    db_name: Option<String>,
    db_port: Option<u32>,
    user: Option<String>,
    passw: Option<String>,
}

fn pull_configuration() -> Figment {
    Figment::new()
    .merge(Toml::file("/../../config/rusty_thermostat.toml"))
    .merge(Env::prefixed("RUSTY_THERMO_"))
}

fn parse_weather(fig: &Figment) -> weather::Configuration {
    //TODO: rip apart the weather part of the figment and make a weather configuration
    weather::Configuration::new()
}

fn parse_db(fig: &Figment) -> dbman::DBConfig {
    //TODO: rip apart the database part of the figment and make a database configuration
    dbman::DBConfig::default()
}

fn parse_logging(fig: &Figment) -> () {
    //TODO: implement full logging and then have this pull apart the logging part of figment to make the logging configuration
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    let fig_settings: Figment = pull_configuration();
    let weather_settings: weather::Configuration = parse_weather(&fig_settings);
    let db_settings: dbman::DBConfig = parse_db(&fig_settings);
    parse_logging(&fig_settings);
    rocket::build().mount("/", routes![index])
}