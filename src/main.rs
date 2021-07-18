#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel_migrations;

use ranklab_api::routes;
use rocket::fairing::AdHoc;
use rocket::figment::providers::Env;
use rocket::serde::{json::Json, Serialize};
use rocket::{Build, Rocket};
use rocket_sync_db_pools::database;

#[database("default")]
struct DbConn(diesel::PgConnection);

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
  embed_migrations!();

  let db_conn = DbConn::get_one(&rocket)
    .await
    .expect("Failed to get db connection");

  db_conn
    .run(|c| embedded_migrations::run(c))
    .await
    .expect("Failed to run migrations");

  rocket
}

#[derive(Serialize)]
struct Health {
  status: String,
}

#[get("/")]
async fn root() -> Json<Health> {
  Json(Health {
    status: "ok".into(),
  })
}

#[launch]
fn rocket() -> Rocket<Build> {
  let mut figment = rocket::Config::figment();

  if let Some(database_url) = Env::var("DATABASE_URL") {
    figment = figment.merge(("databases.default.url", database_url));
  }

  rocket::custom(figment)
    .attach(DbConn::fairing())
    .attach(AdHoc::on_ignite("Run Migrations", run_migrations))
    .mount("/", routes![root])
    .mount("/recordings", routes::recordings())
    .mount("/users", routes::users())
}
