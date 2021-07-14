#[macro_use]
extern crate rocket;

use ranklab_api::routes;
use rocket::figment::providers::Env;
use rocket::serde::{json::Json, Serialize};
use rocket::{Build, Rocket};
use rocket_sync_db_pools::database;

#[derive(Serialize)]
struct Health {
  status: String,
}

#[database("default")]
struct DbConn(diesel::PgConnection);

#[get("/")]
async fn root() -> Json<Health> {
  Json(Health {
    status: "ok".into(),
  })
}

#[launch]
fn rocket() -> Rocket<Build> {
  let mut figment = rocket::Config::figment();

  eprintln!("{}", Env::var("DATABASE_URL").unwrap_or("".into()));

  if let Some(database_url) = Env::var("DATABASE_URL") {
    figment = figment.merge(("databases.default.url", database_url));
  }

  rocket::custom(figment)
    .attach(DbConn::fairing())
    .mount("/", routes![root])
    .mount("/recordings", routes::recordings())
}
