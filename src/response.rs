use okapi::openapi3::Responses;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{response, Request};
use rocket_okapi::{
  gen::OpenApiGenerator, response::OpenApiResponder, util::add_schema_response,
  Result as OpenApiResult,
};
use schemars::JsonSchema;
use serde::Serialize;
use validator::ValidationErrors;

pub enum Response<T: Serialize> {
  Success(T),
  Status(Status),
  ValidationErrors(ValidationErrors),
}

impl<'r, T: Serialize> Responder<'r, 'static> for Response<T> {
  fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
    match self {
      Response::Success(json) => Custom(Status::Ok, Json(json)).respond_to(req),
      Response::Status(status) => status.respond_to(req),
      Response::ValidationErrors(errors) => {
        Custom(Status::UnprocessableEntity, Json(errors)).respond_to(req)
      }
    }
  }
}

impl<T: Serialize + JsonSchema> OpenApiResponder<'_, 'static> for Response<T> {
  fn responses(gen: &mut OpenApiGenerator) -> OpenApiResult<Responses> {
    let mut responses = Responses::default();
    let schema = gen.json_schema::<T>();
    add_schema_response(&mut responses, 200, "application/json", schema)?;
    Ok(responses)
  }
}

impl<T: Serialize> std::ops::FromResidual<diesel::result::QueryResult<std::convert::Infallible>>
  for Response<T>
{
  fn from_residual(residual: diesel::result::QueryResult<std::convert::Infallible>) -> Self {
    match residual {
      Ok(_) => panic!(),
      Err(diesel::result::Error::NotFound) => Response::Status(Status::NotFound),
      Err(err) => {
        sentry::capture_error(&err);
        Response::Status(Status::InternalServerError)
      }
    }
  }
}
