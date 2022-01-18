#[macro_use]
extern crate rocket;

use ranklab_api::config::Config;
use ranklab_api::db::{run_migrations, DbConn};
use ranklab_api::fairings;
use ranklab_api::routes::*;
use rocket::fairing::AdHoc;
use rocket::figment::providers::{Env, Format, Toml};

use rocket::http::Accept;
use rocket::{Build, Rocket};
use rocket_okapi::{openapi, openapi_get_routes};

use rocket::serde::json::Json;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct Health {
  status: String,
}

fn load_envs() {
  let profile = std::env::var("ROCKET_PROFILE").unwrap_or_else(|_| "development".to_string());
  dotenv::from_filename(format!(".env.{}", profile)).ok();
  dotenv::dotenv().ok();
}

#[openapi]
#[get("/")]
pub async fn get_health() -> Json<Health> {
  Json(Health {
    status: "ok".into(),
  })
}

#[launch]
fn rocket() -> Rocket<Build> {
  load_envs();

  let mut figment = rocket::Config::figment()
    .merge(Toml::file("Ranklab.toml").nested())
    .merge(Env::prefixed("RANKLAB_").global());

  if let Some(database_url) = Env::var("DATABASE_URL") {
    figment = figment.merge(("databases.default.url", database_url));
  }

  let sentry_dsn: String = figment.extract_inner("sentry_dsn").unwrap();

  rocket::custom(figment)
    .attach(fairings::Sentry::fairing(sentry_dsn))
    .attach(fairings::Sqs::fairing())
    .attach(DbConn::fairing())
    .attach(AdHoc::on_ignite("Run Migrations", run_migrations))
    .attach(AdHoc::on_request("Accept JSON", |req, _| {
      Box::pin(async move { req.replace_header(Accept::JSON) })
    }))
    .attach(AdHoc::config::<Config>())
    .mount(
      "/",
      openapi_get_routes![
        get_health,
        claims::coaches::create,
        claims::players::create,
        coach::comments::create,
        coach::comments::list,
        coach::comments::update,
        coach::recordings::get,
        coach::reviews::get,
        coach::reviews::list,
        coach::account_links::create,
        player::comments::list,
        player::recordings::create,
        player::recordings::get,
        player::reviews::create,
        player::reviews::get,
        player::reviews::list,
        public::games::list,
        user::users::get_me
      ],
    )
}
