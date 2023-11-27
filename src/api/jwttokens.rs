use jsonwebtoken;
use serde_derive::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::Utc;

// Structure to contain JWT info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JWTClaims {
    pub iss: String,
    pub sub: String,
    pub groups: String,
    pub exp: u64,
}

impl JWTClaims {
    pub fn new(subject: Uuid) -> JWTClaims {
        let mut new_jwt: JWTClaims = JWTClaims::default();
        new_jwt.sub = subject.as_simple().to_string();
        new_jwt
    }
}

impl Default for JWTClaims {
    fn default() -> Self {
        let expiration: u64 = Utc::now()
            .checked_add_signed(chrono::Duration::seconds(900))
            .expect("Invalid timestamp")
            .timestamp()
            .try_into().unwrap_or(jsonwebtoken::get_current_timestamp());
        JWTClaims {
            iss: "Rusty Thermostat".to_string(),
            sub: "DebugSub".to_string(),
            groups: "WebSensor".to_string(),
            exp: expiration }
    }
}