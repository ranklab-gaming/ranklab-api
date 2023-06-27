#[macro_use]
extern crate rocket;

use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_migrations::*;
use ranklab_api::config::Config;
use ranklab_api::guards::DbConn;
use ranklab_api::routes::*;
use ranklab_api::{fairings, oidc};
use rocket::fairing::AdHoc;
use rocket::figment::providers::{Env, Format, Toml};
use rocket::http::Accept;
use rocket::{Build, Rocket};
use rocket_okapi::openapi_get_routes;
use std::env;

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
async fn rocket() -> Rocket<Build> {
  let rocket_profile = match env::var("ROCKET_PROFILE") {
    Ok(profile) => profile,
    Err(_) => rocket::config::Config::DEFAULT_PROFILE.to_string(),
  };

  let env_suffix = match rocket_profile.as_str() {
    "debug" => "development",
    "release" => "production",
    _ => rocket_profile.as_str(),
  };

  dotenv::from_filename(format!(".env.{}", env_suffix)).ok();
  dotenv::dotenv().ok();

  let mut figment = rocket::Config::figment()
    .merge(Toml::file("Ranklab.toml").nested())
    .merge(Env::prefixed("RANKLAB_").global());

  if let Some(database_url) = Env::var("DATABASE_URL") {
    figment = figment.merge(("databases.default.url", database_url));
  }

  let web_host: String = figment.extract_inner("web_host").unwrap();

  rocket::custom(figment)
    .attach(fairings::Sentry::fairing())
    .attach(fairings::Sqs::fairing())
    .attach(DbConn::fairing())
    .attach(AdHoc::on_ignite("Run Migrations", run_migrations))
    .attach(AdHoc::on_request("Accept JSON", |req, _| {
      Box::pin(async move { req.replace_header(Accept::JSON) })
    }))
    .attach(AdHoc::config::<Config>())
    .manage(
      oidc::init_cache(&web_host, &rocket_profile.into())
        .await
        .unwrap(),
    )
    .mount(
      "/",
      openapi_get_routes![
        audios::create,
        audios::delete,
        audios::get,
        avatars::create,
        avatars::delete,
        avatars::get,
        comments::create,
        comments::delete,
        comments::list,
        comments::update,
        games::create,
        games::list,
        passwords::create,
        passwords::update,
        recordings::create,
        recordings::delete,
        recordings::get,
        recordings::list,
        sessions::create,
        users::create,
        users::get,
        users::update,
      ],
    )
}
