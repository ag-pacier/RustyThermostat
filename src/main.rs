use config::{Config, File};
use serde_derive::Deserialize;

pub mod weather;
pub mod schema;
pub mod dbman;

#[macro_use] extern crate rocket;

#[derive(Debug, Deserialize)]
struct AppConfiguration {
    weather_settings: weather::Configuration,
    db_settings: dbman::DBConfig,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    let application_config: Config = Config::builder().add_source(File::with_name("C:/test.toml")).build().unwrap();
    let app_settings: AppConfiguration = application_config.try_deserialize().unwrap();
    rocket::build().mount("/", routes![index])
}