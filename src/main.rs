#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel_migrations;

use ranklab_api::config::Config;
use ranklab_api::fairings;
use ranklab_api::guards::DbConn;
use ranklab_api::routes::*;
use rocket::fairing::AdHoc;
use rocket::figment::providers::{Env, Format, Toml};
use rocket::figment::Profile;
use rocket::http::Accept;
use rocket::{Build, Rocket};
use rocket_okapi::openapi_get_routes;
use schemars::JsonSchema;
use serde::Serialize;
use std::env;

#[derive(Serialize, JsonSchema)]
pub struct Health {
  status: String,
}

pub async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
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

#[launch]
fn rocket() -> Rocket<Build> {
  let profile_name = env::var("ROCKET_PROFILE").unwrap_or("debug".to_string());
  let profile = Profile::new(&profile_name);

  let env_suffix = if profile == rocket::config::Config::DEBUG_PROFILE {
    "development".into()
  } else {
    profile.as_str()
  };

  dotenv::from_filename(format!(".env.{}", env_suffix)).ok();
  dotenv::dotenv().ok();

  let mut figment = rocket::Config::figment()
    .select(profile)
    .merge(Toml::file("Ranklab.toml").nested())
    .merge(Env::prefixed("RANKLAB_").global());

  if let Some(database_url) = Env::var("DATABASE_URL") {
    figment = figment.merge(("databases.default.url", database_url));
  }

  let sentry_dsn: Option<String> = figment.extract_inner("sentry_dsn").ok();

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
        index::get_health,
        claims::coaches::create,
        claims::players::create,
        claims::coaches::available_countries,
        coach::comments::create,
        coach::comments::list,
        coach::comments::update,
        coach::recordings::get,
        coach::reviews::get,
        coach::reviews::list,
        coach::reviews::update,
        coach::stripe_account_links::create,
        coach::stripe_login_links::create,
        player::comments::list,
        player::recordings::create,
        player::recordings::get,
        player::recordings::list,
        player::reviews::get,
        player::reviews::list,
        player::reviews::create,
        player::reviews::update,
        player::stripe_billing_portal_sessions::create,
        player::stripe_payment_methods::list,
        public::games::list,
        user::users::get_me
      ],
    )
}
