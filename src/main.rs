#[macro_use]
extern crate rocket;

use ranklab_api::config::Config;
use ranklab_api::db::{run_migrations, DbConn};
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

#[openapi]
#[get("/")]
pub async fn get_health() -> Json<Health> {
  Json(Health {
    status: "ok".into(),
  })
}

#[launch]
fn rocket() -> Rocket<Build> {
  let _guard = sentry::init((
    "https://c7b459471051450abcfb5b4e25fa2b2c@o1059892.ingest.sentry.io/6048906",
    sentry::ClientOptions {
      release: sentry::release_name!(),
      ..Default::default()
    },
  ));

  let mut figment = rocket::Config::figment()
    .merge(Toml::file("Ranklab.toml").nested())
    .merge(Env::prefixed("RANKLAB_").global());

  if let Some(database_url) = Env::var("DATABASE_URL") {
    figment = figment.merge(("databases.default.url", database_url));
  }

  rocket::custom(figment)
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
        users::get_current,
        recordings::create,
        reviews::get,
        reviews::list,
        reviews::create,
        coaches::create,
        comments::create,
        games::list,
        comments::list
      ],
    )
}
