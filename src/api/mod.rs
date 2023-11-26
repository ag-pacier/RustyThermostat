use rocket::response::status;
use uuid::Uuid;

#[get("/api")]
pub fn api_index() -> &'static str {
    "Hello, API!"
}

#[get("/api/weather")]
pub fn weather() -> &'static str {
    "Current weather"
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
