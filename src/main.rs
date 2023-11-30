use figment::{Figment, providers::{Format, Toml, Env}};
use serde_derive::Deserialize;

pub mod weather;
pub mod schema;
pub mod dbman;

#[macro_use] extern crate rocket;

#[derive(Debug, Deserialize)]
struct AppConfiguration {
    rocket_settings: rocket::config::Config,
    weather_settings: weather::Configuration,
    db_settings: dbman::DBConfig,
}

fn pull_configuration() -> Figment {
    Figment::new()
    .merge(Toml::file("/../../config/rusty_thermostat.toml"))
    .merge(Env::prefixed("RUSTY_THERMO_"))
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}