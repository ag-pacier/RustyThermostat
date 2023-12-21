// Library for containing database logic

use log;
use serde_derive::Deserialize;
use std::{fmt, time::Duration};
use sea_orm::{Database, ConnectOptions, DatabaseConnection, DbErr};

// Structure for the database
// Default will create a SQLite in-memory DB with debug logging
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct DBConfig {
    pub protocol: String,
    pub port: Option<u32>,
    pub dbhost: String,
    pub db_db: Option<String>,
    pub acct: Option<String>,
    pub pass: Option<String>,
    pub con_min: Option<u32>,
    pub con_max: Option<u32>,
    pub timeout_con: Option<u64>,
    pub timeout_acq: Option<u64>,
    pub timeout_idle: Option<u64>,
    pub max_life: Option<u64>,
    pub log: bool,
    pub log_level: log::LevelFilter,
    pub schema_path: Option<String>,
}

impl DBConfig {
    /// Creates a DBConfig that makes assumptions based on a postgresql setup
    /// dbhost needs to be the host running the postgresql software
    /// user & pass must be valid users with sufficent privledges to perform all application functions
    /// db needs to be the specific database created for the application
    /// If needed, this can accept a schema path and a port number
    /// Port will default to PostgreSQL default of 5432/TCP
    pub fn new_postgres(dbhost: String, user: String, pass: String, db: String, schema_path: Option<String>, port: Option<u32>,) -> DBConfig {
        debug!("Creating DBConfig based on PostgreSQL");
        debug!("Host: {}, DB: {},", &dbhost, &db);
        trace!("User: {}, Password: {}", &user, &pass);
        if schema_path.is_some() {
            trace!("Schema path set to: {}", schema_path.clone().unwrap());
        }
        if port.is_some() {
            debug!("Port set to: {}", port.clone().unwrap());
        }
        DBConfig {
            protocol: "postgres://".to_string(),
            port: Some(port.unwrap_or(5432)),
            dbhost: dbhost,
            db_db: Some(db),
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

    /// Creates a DBConfig that makes assumptions based on a SQLite setup
    /// dbpath is expecting a path that starts with either the drive letter or the first folder closest to root
    pub fn new_sqlite(dbpath: String) -> DBConfig {
        debug!("Creating DBConfig based on SQLITE");
        let mut new_db: DBConfig = DBConfig::default();
        new_db.protocol = "sqlite://".to_string();
        new_db.dbhost = format!("{}?mode=rwc", dbpath);
        new_db
    }

    /// Generates the URI based on what is available in the DBConfig
    fn build_connect_string(&self) -> String {
        let local_config: DBConfig = self.clone();
        let mut connect_string: String = self.protocol.clone();
        if local_config.acct.is_some() {
            connect_string = format!("{}{}:{}@", connect_string, local_config.acct.unwrap(), local_config.pass.unwrap());
        }
        connect_string = format!("{}{}", connect_string, local_config.dbhost);
        if let Some(port_choice) = local_config.port {
            connect_string = format!("{}:{}", connect_string, port_choice);
        }
        if let Some(database_set) = local_config.db_db {
            connect_string = format!("{}/{}", connect_string, database_set);
        }
        if let Some(schema_choice) = local_config.schema_path {
            connect_string = format!("{}?currentSchema={}", connect_string, schema_choice);
        }

        debug!("Connection string built: {}", &connect_string);
        connect_string
    }

    /// Returns a String showing if logging is on and what level
    pub fn check_logging(&self) -> String {
        let log_level: &str = match self.log_level {
            log::LevelFilter::Off => "OFF",
            log::LevelFilter::Error => "Error only",
            log::LevelFilter::Warn => "Warn/Error only",
            log::LevelFilter::Info => "Info/Warn/Error",
            log::LevelFilter::Debug => "Debug",
            log::LevelFilter::Trace => "TRACE",
        };
        format!("Logging enabled: {} and set to: {}", self.log, log_level)
    }

    // Generates the connection options based on the DBConfig
    pub fn set_connect_options(&self) -> ConnectOptions {
        let local_config: DBConfig = self.clone();
        let mut opt = ConnectOptions::new(local_config.build_connect_string());
        if let Some(con_max) = local_config.con_max {
            debug!("Set max connections to: {}", con_max.clone());
            opt.max_connections(con_max);
        }
        if let Some(con_min) = local_config.con_min {
            debug!("Set min connections to: {}", con_min.clone());
            opt.min_connections(con_min);
        }
        if let Some(con_time) = local_config.timeout_con {
            debug!("Set connection timeout to: {}", con_time.clone());
            opt.connect_timeout(Duration::from_secs(con_time));
        }
        if let Some(acq_time) = local_config.timeout_acq {
            debug!("Set acquire timeout to: {}", acq_time.clone());
            opt.acquire_timeout(Duration::from_secs(acq_time));
        }
        if let Some(idle_time) = local_config.timeout_idle {
            debug!("Set idle timeout to: {}", idle_time.clone());
            opt.idle_timeout(Duration::from_secs(idle_time));
        }
        if let Some(lifetime) = local_config.max_life {
            debug!("Set max lifetime to: {}", lifetime.clone());
            opt.max_lifetime(Duration::from_secs(lifetime));
        }
        if let Some(schemapath) = local_config.schema_path {
            debug!("Set schema path to: {}", schemapath.clone());
            opt.set_schema_search_path(schemapath);
        }
        if local_config.log {
            debug!("Set SQLX logging to true!");
            opt.sqlx_logging(true);
        } else {
            debug!("Set SQLX logging to false!");
            opt.sqlx_logging(false);
        }
        opt.sqlx_logging_level(local_config.log_level.clone());
        debug!("SQLX logging level set to: {}", local_config.log_level);

        opt
    }

}

impl fmt::Display for DBConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build_connect_string())
    }
}

impl Default for DBConfig {
    fn default() -> Self {
        DBConfig {
            protocol: "sqlite".to_string(),
            port: None,
            dbhost: "::memory:".to_string(),
            db_db: None,
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

pub async fn begin_connection(con_settings: ConnectOptions) -> Result<DatabaseConnection, DbErr> {
    Database::connect(con_settings).await
}

pub async fn is_live(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.ping().await
}