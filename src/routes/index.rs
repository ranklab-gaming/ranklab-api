use rocket::serde::json::Json;
use rocket::Route;
use rocket_okapi::{openapi, openapi_get_routes as routes};
use schemars::JsonSchema;
use serde::Serialize;

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
