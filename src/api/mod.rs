

#[get("/api")]
pub fn api_index() -> &'static str {
    "Hello, API!"
}