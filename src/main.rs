use figment::{Figment, providers::{Format, Toml, Env}};
use serde_derive::Deserialize;
use simplelog;
use tokio::fs::File;

pub mod weather;
pub mod schema;
pub mod dbman;

#[macro_use] extern crate log;
#[macro_use] extern crate rocket;

#[derive(Clone, Debug, Deserialize)]
struct AppConfiguration {
    weather: WeatherSettings,
    database: DatabaseSettings,
    logging: LogSettings,
}

impl Default for AppConfiguration {
    fn default() -> Self {
        AppConfiguration {
            weather: WeatherSettings::default(),
            database: DatabaseSettings::default(),
            logging: LogSettings::default()
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct LogSettings {
    enabled: bool,
    log_level: Option<String>,
    log_location: Option<String>
}

impl Default for LogSettings {
    fn default() -> Self {
        LogSettings {
            enabled: true,
            log_level: Some("debug".to_string()),
            log_location: Some("./rusty.log".to_string())
        }
    }
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

impl WeatherSettings {
    fn is_active(&self) -> bool {
        if self.monitor_weather {
            return true
        } else if self.monitor_pollution {
            return true
        } else {
            return false
        }
    }
}

impl Default for WeatherSettings {
    fn default() -> Self {
        WeatherSettings {
            monitor_weather: false,
            monitor_pollution: false,
            zip_code: "N/A".to_string(),
            country: None,
            units: None,
            openweather_apikey: None
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct DatabaseSettings {
    database_type: String,
    host: String,
    db_name: Option<String>,
    db_port: Option<u32>,
    user: Option<String>,
    passw: Option<String>,
    schema_path: Option<String>,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        DatabaseSettings {
            database_type: "default".to_string(),
            host: "127.0.0.1".to_string(),
            db_name: None,
            db_port: None,
            user: None,
            passw: None,
            schema_path: None,
        }
    }
}

fn pull_configuration() -> AppConfiguration {
    let fig_config: Figment = Figment::new()
    .merge(Toml::file("/../../config/rusty_thermostat.toml"))
    .merge(Env::prefixed("RUSTY_THERMO_"));

    match fig_config.extract() {
        Ok(extracted) => extracted,
        Err(_) => AppConfiguration::default()
    }
}

async fn parse_weather(fig: &AppConfiguration) -> Result<weather::Configuration, weather::APIError> {
    let mut wea_config = weather::Configuration::new();
    let wea_part: WeatherSettings = fig.weather.clone();
    if let Some(set_units) = wea_part.units {
        wea_config.set_units(&set_units);
    }
    if let Some(key) = wea_part.openweather_apikey {
        wea_config.api_key = Some(key);
    }

    let mut zip_local = String::from(wea_part.zip_code);
    if let Some(country) = wea_part.country {
        zip_local = format!("{},{}", zip_local, country);
    }
    wea_config.location = Some(wea_config.parse_zipcode(&zip_local).await?);

    Ok(wea_config)
}

fn parse_db(fig: &AppConfiguration) -> dbman::DBConfig {
    let mut data_config: dbman::DBConfig = dbman::DBConfig::default();
    let local_db_sets: DatabaseSettings = fig.database.clone();

    let data_type: &str = match local_db_sets.database_type.as_str() {
        "postgres" => "postgres",
        "postgresql" => "postgres",
        "sqlite" => "sqlite",
        _ => "UNKNOWN",
    };

    if data_type == "postgres" {
        data_config = dbman::DBConfig::new_postgres(
            local_db_sets.host,
            local_db_sets.user.unwrap(),
            local_db_sets.passw.unwrap(),
            local_db_sets.db_name.unwrap_or("rusty_thermostat".to_string()),
            local_db_sets.schema_path,
            local_db_sets.db_port);
    } else if data_type == "sqlite" {
        data_config = dbman::DBConfig::new_sqlite(local_db_sets.host);
        
    }

    data_config
}

fn parse_log(fig: &AppConfiguration) -> () {
    let log_sets: LogSettings = fig.logging.clone();
    
    let term_logger: Box<simplelog::TermLogger> = simplelog::TermLogger::new(
            simplelog::LevelFilter::Error,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto);
    if log_sets.enabled {

    } else {

    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
async fn rocket() -> _ {
    let runtime_settings: AppConfiguration = pull_configuration();
    let _weather_settings: weather::Configuration = match runtime_settings.weather.is_active() {
        true => parse_weather(&runtime_settings).await.unwrap(),
        false => weather::Configuration::default()
    };
    let _db_settings: dbman::DBConfig = parse_db(&runtime_settings);
    rocket::build().mount("/", routes![index])
}