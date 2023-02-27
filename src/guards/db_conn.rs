use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
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
