#[macro_use]
extern crate rocket;

use ranklab_api::routes;
use rocket::serde::{json::Json, Serialize};
use rocket::{Build, Rocket};

#[derive(Serialize)]
struct Health {
  status: String,
}

#[get("/")]
fn root() -> Json<Health> {
  Json(Health {
    status: "ok".to_string(),
  })
}

#[launch]
fn rocket() -> Rocket<Build> {
  rocket::build()
    .mount("/", routes![root])
    .mount("/recordings", routes::recordings())
}
