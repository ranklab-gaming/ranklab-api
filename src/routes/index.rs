use rocket::serde::json::Json;
use rocket::Route;
use serde::Serialize;
use rocket_okapi::{openapi, openapi_get_routes as routes, JsonSchema};

#[derive(Serialize, JsonSchema)]
struct Health {
    status: String,
}

#[openapi]
#[get("/")]
async fn get_health() -> Json<Health> {
    Json(Health {
        status: "ok".into(),
    })
}

pub fn build() -> Vec<Route> {
    routes![get_health]
}
