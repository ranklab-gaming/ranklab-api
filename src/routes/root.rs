use rocket::serde::json::Json;
use rocket::Route;
use serde::Serialize;

#[derive(Serialize)]
struct Health {
  status: String,
}

#[get("/")]
async fn get_health() -> Json<Health> {
  Json(Health {
    status: "ok".into(),
  })
}

pub fn build() -> Vec<Route> {
  routes![get_health]
}
