use okapi::openapi3::Responses;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{response, Request};
use rocket_okapi::{
  gen::OpenApiGenerator, response::OpenApiResponderInner, Result as OpenApiResult,
};
use validator::ValidationErrors;

fn add_400_error(responses: &mut Responses) {
  responses
    .responses
    .entry("400".to_owned())
    .or_insert_with(|| {
      let response = okapi::openapi3::Response {
        description:
          "# [400 Bad Request](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400)"
            .to_owned(),
        ..Default::default()
      };
      response.into()
    });
}

fn add_401_error(responses: &mut Responses) {
  responses
    .responses
    .entry("401".to_owned())
    .or_insert_with(|| {
      let response = okapi::openapi3::Response {
        description:
          "# [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)"
            .to_owned(),
        ..Default::default()
      };
      response.into()
    });
}

fn add_404_error(responses: &mut Responses) {
  responses
    .responses
    .entry("404".to_owned())
    .or_insert_with(|| {
      let response = okapi::openapi3::Response {
        description:
          "# [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)"
            .to_owned(),
        ..Default::default()
      };
      response.into()
    });
}

fn add_422_error(responses: &mut Responses) {
  responses
    .responses
    .entry("422".to_owned())
    .or_insert_with(|| {
      let response = okapi::openapi3::Response {
        description:
          "# [422 Unprocessable Entity](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/422)"
            .to_owned(),
        ..Default::default()
      };
      response.into()
    });
}

fn add_500_error(responses: &mut Responses) {
  responses
    .responses
    .entry("500".to_owned())
    .or_insert_with(|| {
      let response = okapi::openapi3::Response {
        description:
          "# [500 Internal Server Error](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/500)"
            .to_owned(),
        ..Default::default()
      };
      response.into()
    });
}

pub enum MutationError {
  ValidationErrors(ValidationErrors),
  Status(Status),
}

pub enum QueryError {
  Status(Status),
}

pub type MutationResponse<T> = Result<Json<T>, MutationError>;
pub type QueryResponse<T> = Result<Json<T>, QueryError>;
pub struct Response;

impl Response {
  pub fn success<T, E>(response: T) -> Result<Json<T>, E> {
    Ok(Json(response))
  }

  pub fn query_error<T>(status: Status) -> Result<Json<T>, QueryError> {
    Err(QueryError::Status(status))
  }

  pub fn validation_error<T>(errors: ValidationErrors) -> Result<Json<T>, MutationError> {
    Err(MutationError::ValidationErrors(errors))
  }

  pub fn mutation_error<T, E>(status: Status) -> Result<Json<T>, MutationError> {
    Err(MutationError::Status(status))
  }
}

impl<'r> Responder<'r, 'static> for MutationError {
  fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
    match self {
      MutationError::Status(status) => status.respond_to(req),
      MutationError::ValidationErrors(errors) => {
        Custom(Status::UnprocessableEntity, Json(errors)).respond_to(req)
      }
    }
  }
}

impl<'r> Responder<'r, 'static> for QueryError {
  fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
    match self {
      QueryError::Status(status) => status.respond_to(req),
    }
  }
}

impl OpenApiResponderInner for MutationError {
  fn responses(_gen: &mut OpenApiGenerator) -> OpenApiResult<Responses> {
    let mut responses = Responses::default();
    add_400_error(&mut responses);
    add_401_error(&mut responses);
    add_404_error(&mut responses);
    add_422_error(&mut responses);
    add_500_error(&mut responses);
    Ok(responses)
  }
}

impl OpenApiResponderInner for QueryError {
  fn responses(_gen: &mut OpenApiGenerator) -> OpenApiResult<Responses> {
    let mut responses = Responses::default();
    add_400_error(&mut responses);
    add_401_error(&mut responses);
    add_404_error(&mut responses);
    add_500_error(&mut responses);
    Ok(responses)
  }
}

impl From<diesel::result::Error> for MutationError {
  fn from(error: diesel::result::Error) -> Self {
    match error {
      diesel::result::Error::NotFound => MutationError::Status(Status::NotFound),
      _ => MutationError::Status(Status::InternalServerError),
    }
  }
}

impl From<diesel::result::Error> for QueryError {
  fn from(error: diesel::result::Error) -> Self {
    match error {
      diesel::result::Error::NotFound => QueryError::Status(Status::NotFound),
      _ => QueryError::Status(Status::InternalServerError),
    }
  }
}
