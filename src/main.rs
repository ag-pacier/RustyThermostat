use rocket::figment::providers::{Toml, Format, Env};
use rocket::State;
use sea_orm::DatabaseConnection;
use serde_derive::Deserialize;

pub mod weather;
pub mod schema;
pub mod dbman;
pub mod api;

#[macro_use] extern crate rocket;
#[macro_use] extern crate log;

#[derive(Clone, Debug, Deserialize)]
struct AppConfiguration {
    weather: WeatherSettings,
    database: DatabaseSettings,
    logging: LogSettings
}

impl Default for AppConfiguration {
    fn default() -> Self {
        trace!("Using default app config!");
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
            trace!("Monitor weather set to true");
            return true
        } else if self.monitor_pollution {
            trace!("Monitor pollution set to true");
            return true
        } else {
            trace!("Neither weather or pollution being monitored");
            return false
        }
    }
}

impl Default for WeatherSettings {
    fn default() -> Self {
        trace!("Using default weather config!");
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
        trace!("Using default DB config!");
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

async fn parse_weather(fig: &AppConfiguration) -> Result<weather::Configuration, weather::APIError> {
    let mut wea_config: weather::Configuration = weather::Configuration::new();
    let wea_part: WeatherSettings = fig.weather.clone();
    if let Some(set_units) = wea_part.units {
        wea_config.set_units(&set_units);
        debug!("Set units: {}", set_units);
    }
    if let Some(key) = wea_part.openweather_apikey {
        wea_config.api_key = Some(key.clone());
        debug!("Found APIKEY setting.");
        trace!("API key detected as: {}", key);
    }

    let mut zip_local: String = String::from(wea_part.zip_code);
    debug!("Found ZIP of: {}", zip_local);
    if let Some(country) = wea_part.country {
        zip_local = format!("{},{}", zip_local, country);
        debug!("Found country setting of: {}", country);
        debug!("Zip now reads as: {}", zip_local);
    }
    wea_config.location = Some(wea_config.parse_zipcode(&zip_local).await?);

    Ok(wea_config)
}

fn parse_db(fig: &AppConfiguration) -> dbman::DBConfig {
    let mut data_config: dbman::DBConfig = dbman::DBConfig::default();
    let local_db_sets: DatabaseSettings = fig.database.clone();
    trace!("DB settings found as: {:#?}", &local_db_sets);

    let data_type: &str = match local_db_sets.database_type.as_str() {
        "postgres" => "postgres",
        "postgresql" => "postgres",
        "sqlite" => "sqlite",
        _ => "UNKNOWN",
    };

    if data_type == "postgres" {
        trace!("postgres database being set.");
        data_config = dbman::DBConfig::new_postgres(
            local_db_sets.host,
            local_db_sets.user.unwrap(),
            local_db_sets.passw.unwrap(),
            local_db_sets.db_name.unwrap_or("rusty_thermostat".to_string()),
            local_db_sets.schema_path,
            local_db_sets.db_port);
    } else if data_type == "sqlite" {
        trace!("sqlite database being set.");
        data_config = dbman::DBConfig::new_sqlite(local_db_sets.host);  
    }

    data_config
}

fn parse_log(fig: &AppConfiguration) -> () {
    let log_sets: LogSettings = fig.logging.clone();
    let mut log_leveling: simplelog::LevelFilter = simplelog::LevelFilter::Error;
    if log_sets.log_level.is_some() {
        match log_sets.log_level.unwrap().to_lowercase().as_str() {
            "debug" => log_leveling = simplelog::LevelFilter::Debug,
            "info" => log_leveling = simplelog::LevelFilter::Info,
            "error" => log_leveling = simplelog::LevelFilter::Error,
            "warn" => log_leveling = simplelog::LevelFilter::Warn,
            "trace" => log_leveling = simplelog::LevelFilter::Trace,
            "off" => log_leveling = simplelog::LevelFilter::Off,
            _ => log_leveling = simplelog::LevelFilter::Error,
        }
    }
    let logging_configuration = simplelog::ConfigBuilder::new()
        .set_time_format_rfc3339().build();
    if log_sets.enabled {
    let file_logger: Box<simplelog::WriteLogger<std::fs::File>> = simplelog::WriteLogger::new(
        log_leveling,
        logging_configuration.clone(),
        std::fs::File::create(log_sets.log_location.unwrap_or("./rusty.log".to_string())).unwrap());
    let term_logger: Box<simplelog::TermLogger> = simplelog::TermLogger::new(
        simplelog::LevelFilter::Error,
        logging_configuration.clone(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto);
    simplelog::CombinedLogger::init(vec![file_logger, term_logger]).unwrap();
    } else {
    let _ = simplelog::TermLogger::init(log_leveling, logging_configuration.clone(), simplelog::TerminalMode::Stderr, simplelog::ColorChoice::Auto);
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/dbping")]
async fn db_ping(db: &State<DatabaseConnection>) -> &'static str {
    let db: &DatabaseConnection = db as &DatabaseConnection;
    match dbman::is_live(db).await {
        Ok(()) => "Db looks live.",
        Err(_) => "DBPing did not work.",
    }

}

#[launch]
async fn rocket() -> _ {
    let figment: rocket::figment::Figment = rocket::Config::figment()
        .merge(Toml::file("config/rusty_thermostat.toml"))
        .merge(Env::prefixed("RUSTY_THERMO_"));

    let runtime_settings: AppConfiguration = figment.clone().extract().unwrap();
    parse_log(&runtime_settings);
    info!("Logging has been enabled");
    let _weather_settings: weather::Configuration = match runtime_settings.weather.is_active() {
        true => parse_weather(&runtime_settings).await.unwrap(),
        false => weather::Configuration::default()
    };
    let db_settings: dbman::DBConfig = parse_db(&runtime_settings);
    let db_options: sea_orm::ConnectOptions = db_settings.set_connect_options();
    let db: sea_orm::prelude::DatabaseConnection = dbman::begin_connection(db_options).await.unwrap();
    match dbman::is_live(&db).await {
        Ok(()) => info!("Db looks live."),
        Err(_) => error!("DBPing did not work."),
    };
    info!("Setting parsing complete. Starting web server now.");
    rocket::build().configure(figment).manage(db).mount("/", routes![index,
        db_ping,
        api::api_index,
        api::weather,
        api::sensor_status,
        api::zone_status,
        api::pollution])
}