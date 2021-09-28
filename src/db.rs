use rocket::{Build, Rocket};
use rocket_okapi::{
  gen::OpenApiGenerator,
  request::{OpenApiFromRequest, RequestHeaderInput},
};
use rocket_sync_db_pools::database;

#[database("default")]
pub struct DbConn(diesel::PgConnection);

impl<'a> OpenApiFromRequest<'a> for DbConn {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::None)
  }
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
