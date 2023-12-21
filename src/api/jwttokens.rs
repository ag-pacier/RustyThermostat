use jsonwebtoken::{self, Algorithm, Header, EncodingKey, encode, decode, DecodingKey, Validation};
use jsonwebtoken::errors::Error;
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
    pub fn new_sensor(subject: Uuid, sec_token: [u8; 32]) -> Result<String, Error> {
        let mut new_jwt: JWTClaims = JWTClaims::default();
        new_jwt.sub = subject.as_simple().to_string();
        new_jwt.groups = "WebSensor".to_string();
        new_jwt.exp = Utc::now()
            .checked_add_signed(chrono::Duration::seconds(900))
            .expect("Invalid timestamp")
            .timestamp()
            .try_into().unwrap_or(jsonwebtoken::get_current_timestamp());
        let header = Header::new(Algorithm::HS512);
        encode(&header, &new_jwt, &EncodingKey::from_secret(&sec_token))
    }

    pub fn new_collector(subject: Uuid, sec_token: [u8; 32]) -> Result<String, Error> {
        let mut new_jwt: JWTClaims = JWTClaims::default();
        new_jwt.sub = subject.as_simple().to_string();
        new_jwt.groups = "Collector".to_string();
        new_jwt.exp = Utc::now()
            .checked_add_signed(chrono::Duration::seconds(900))
            .expect("Invalid timestamp")
            .timestamp()
            .try_into().unwrap_or(jsonwebtoken::get_current_timestamp());
        let header = Header::new(Algorithm::HS512);
        encode(&header, &new_jwt, &EncodingKey::from_secret(&sec_token))
    }

    pub fn new_viewer(subject: Uuid, sec_token: [u8; 32]) -> Result<String, Error> {
        let mut new_jwt: JWTClaims = JWTClaims::default();
        new_jwt.sub = subject.as_simple().to_string();
        new_jwt.groups = "ViewOnly".to_string();
        new_jwt.exp = Utc::now()
            .checked_add_signed(chrono::Duration::seconds(900))
            .expect("Invalid timestamp")
            .timestamp()
            .try_into().unwrap_or(jsonwebtoken::get_current_timestamp());
        let header = Header::new(Algorithm::HS512);
        encode(&header, &new_jwt, &EncodingKey::from_secret(&sec_token))
    }

    pub fn new_admin(subject: Uuid, sec_token: [u8; 32]) -> Result<String, Error> {
        let mut new_jwt: JWTClaims = JWTClaims::default();
        new_jwt.sub = subject.as_simple().to_string();
        new_jwt.groups = "Super".to_string();
        new_jwt.exp = Utc::now()
            .checked_add_signed(chrono::Duration::seconds(900))
            .expect("Invalid timestamp")
            .timestamp()
            .try_into().unwrap_or(jsonwebtoken::get_current_timestamp());
        let header = Header::new(Algorithm::HS512);
        encode(&header, &new_jwt, &EncodingKey::from_secret(&sec_token))
    }
}

impl Default for JWTClaims {
    fn default() -> Self {
        let expiration: u64 = jsonwebtoken::get_current_timestamp();
        JWTClaims {
            iss: "Rusty Thermostat".to_string(),
            sub: "DebugSub".to_string(),
            groups: "NONE".to_string(),
            exp: expiration }
    }
}

pub fn decode_claim(token: String, sec_token: [u8; 32]) -> Result<JWTClaims, Error> {
    let decoded_token: jsonwebtoken::TokenData<JWTClaims> = decode::<JWTClaims>(&token, &DecodingKey::from_secret(&sec_token), &Validation::default())?;
    Ok(decoded_token.claims)
}