// Library for containing database logic

use log;
use std::time::Duration;
use sea_orm::ConnectOptions;

// Structure for the database
// Default will create a SQLite in-memory DB with debug logging
#[derive(Clone, Debug)]
struct DBConfig {
    protocol: String,
    port: Option<i32>,
    dbpath: String,
    acct: Option<String>,
    pass: Option<String>,
    con_min: Option<i32>,
    con_max: Option<i32>,
    timeout_con: Option<i32>,
    timeout_acq: Option<i32>,
    timeout_idle: Option<i32>,
    max_life: Option<i32>,
    log: bool,
    log_level: log::LevelFilter,
    schema_path: Option<String>,
}

impl DBConfig {
    pub fn new_postgres(dbpath: String, user: String, pass: String, schema_path: Option<String>, port: Option<i32>,) -> DBConfig {
        DBConfig {
            protocol: "postgres://".to_string(),
            port: Some(port.unwrap_or(5432)),
            dbpath: dbpath,
            acct: Some(user),
            pass: Some(pass),
            con_min: None,
            con_max: None,
            timeout_con: None,
            timeout_acq: None,
            timeout_idle: None,
            max_life: None,
            log: true,
            log_level: log::LevelFilter::Debug,
            schema_path: schema_path }
    }

    pub fn new_sqlite(dbpath: String) -> DBConfig {
        let mut new_db: DBConfig = DBConfig::default();
        new_db.protocol = "sqlite://".to_string();
        new_db.dbpath = format!("{}?mode=rwc", dbpath);
        new_db
    }

    pub fn build_connect_string(&self) -> String {
        let local_config: DBConfig = self.clone();
        let mut connect_string: String = self.protocol.clone();
        if local_config.acct.is_some() {
            connect_string = format!("{}{}:{}@", connect_string, local_config.acct.unwrap(), local_config.pass.unwrap());
        }
        format!("{}{}", connect_string, local_config.dbpath)
    }

    pub fn set_connect_options(&self) -> ConnectOptions {
        let local_config: DBConfig = self.clone();
        let mut opt = ConnectOptions::new(local_config.build_connect_string());
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Info)
            .set_schema_search_path("my_schema");

        opt
    }
}

impl Default for DBConfig {
    fn default() -> Self {
        DBConfig {
            protocol: "sqlite".to_string(),
            port: None,
            dbpath: "::memory:".to_string(),
            acct: None,
            pass: None,
            con_min: None,
            con_max: None,
            timeout_con: None,
            timeout_acq: None,
            timeout_idle: None,
            max_life: None,
            log: true,
            log_level: log::LevelFilter::Debug,
            schema_path: None }
    }
}