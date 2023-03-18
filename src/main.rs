#[macro_use]
extern crate rocket;

use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_migrations::*;
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
use std::env;

const DEFAULT_PROFILE: Profile = rocket::config::Config::DEFAULT_PROFILE;
const DEBUG_PROFILE: Profile = rocket::config::Config::DEBUG_PROFILE;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
  let database_url: String = rocket
    .figment()
    .extract_inner("databases.default.url")
    .unwrap();

  let mut conn = PgConnection::establish(&database_url).unwrap();

  conn.run_pending_migrations(MIGRATIONS).unwrap();

  rocket
}

#[launch]
fn rocket() -> Rocket<Build> {
  let env_suffix = if DEFAULT_PROFILE == DEBUG_PROFILE {
    "development".to_owned()
  } else {
    DEFAULT_PROFILE.to_string()
  };

  dotenv::from_filename(format!(".env.{}", env_suffix)).ok();
  dotenv::dotenv().ok();

  let mut figment = rocket::Config::figment()
    .merge(Toml::file("Ranklab.toml").nested())
    .merge(Env::prefixed("RANKLAB_").global());

  if let Some(database_url) = Env::var("DATABASE_URL") {
    figment = figment.merge(("databases.default.url", database_url));
  }

  rocket::custom(figment)
    .attach(fairings::Sentry::fairing())
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
        coach::account::create,
        coach::account::get,
        coach::account::update,
        coach::comments::create,
        coach::comments::list,
        coach::comments::update,
        coach::reviews::get,
        coach::reviews::list,
        coach::reviews::update,
        coach::stripe_account_links::create,
        coach::stripe_country_specs::list,
        coach::stripe_login_links::create,
        game::list,
        index::get_health,
        player::account::create,
        player::account::get,
        player::account::update,
        player::coaches::list,
        player::comments::list,
        player::recordings::create,
        player::recordings::get,
        player::recordings::list,
        player::reviews::create,
        player::reviews::delete,
        player::reviews::get,
        player::reviews::list,
        player::reviews::update,
        player::stripe_billing_portal_sessions::create,
        player::stripe_payment_methods::list,
        player::stripe_tax_calculations::create,
        session::create,
        session::reset_password,
        session::update_password
      ],
    )
}
