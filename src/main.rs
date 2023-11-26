pub mod weather;
pub mod schema;
pub mod api;

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index,
        api::api_index,
        api::weather,
        api::sensor_status,
        api::zone_status,
        api::pollution])
}