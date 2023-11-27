use rocket::response::status;
use rocket::serde::json::Json;
use uuid::Uuid;
use crate::weather::WeatherResponse;

// Supporting functions to generate content for API pages goes here
fn test_weather() -> WeatherResponse {
    WeatherResponse::test_my_weather()
}

// Routes go below here for the API
// GET routes
#[get("/api")]
pub fn api_index() -> &'static str {
    "Hello, API!"
}

#[get("/api/weather")]
pub fn weather() -> Json<WeatherResponse> {
    Json(test_weather())
}

#[get("/api/pollution")]
pub fn pollution() -> &'static str {
    "Current pollution"
}

#[get("/api/sensor/<id>")]
pub fn sensor_status(id: &str) -> Result<String, status::BadRequest<String>> {
    let parsed_uuid: Result<Uuid, uuid::Error> = Uuid::parse_str(id);
    if parsed_uuid.is_err() {
        Err(status::BadRequest(format!("The provided ID: {} cannot become a UUID.", id)))
    } else {
        Ok(format!("The ID provided: {} can be seen like this: {}", id, parsed_uuid.unwrap().as_hyphenated().to_string()))
    }
}

#[get("/api/controller/<id>")]
pub fn controller_status(id: &str) -> Result<String, status::BadRequest<String>> {
    let parsed_uuid: Result<Uuid, uuid::Error> = Uuid::parse_str(id);
    if parsed_uuid.is_err() {
        Err(status::BadRequest(format!("The provided ID: {} cannot become a UUID.", id)))
    } else {
        Ok(format!("The ID provided: {} can be seen like this: {}", id, parsed_uuid.unwrap().as_hyphenated().to_string()))
    }
}

#[get("/api/zone/<id>")]
pub fn zone_status(id: u32) -> Result<String, status::BadRequest<String>> {
    Ok(format!("Info for zone {}:", id))
}

// PUT routes

// PATCH routes

// DELETE routes
