use crate::models::{CoachInvitation, OneTimeToken};
use okapi::openapi3::*;
use rocket::request::{FromRequest, Outcome, Request};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
mod jwt;
mod ott;
pub use jwt::{FromJwt, Jwt};
pub use ott::{CoachInvitationParams, OneTimeTokenParams};
use rocket::http::Status;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
  #[error("missing authorization header")]
  Missing,
  #[error("invalid token: {0}")]
  Invalid(String),
  #[error("not found: {0}")]
  NotFound(String),
}

impl From<AuthError> for (Status, AuthError) {
  fn from(error: AuthError) -> Self {
    match error {
      AuthError::Missing => (Status::Unauthorized, error),
      AuthError::Invalid(_) => (Status::BadRequest, error),
      AuthError::NotFound(_) => (Status::NotFound, error),
    }
  }
}

#[async_trait]
pub trait AuthFromRequest
where
  Self: Sized,
{
  async fn from_request(request: &Request<'_>) -> Result<Self, AuthError>;
}

pub struct Auth<T>(pub T);

impl<T> Auth<T> {
  pub fn into_inner(self) -> T {
    self.0
  }
}

#[async_trait]
impl<'r, T: AuthFromRequest> FromRequest<'r> for Auth<T> {
  type Error = AuthError;

  async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    match T::from_request(request).await {
      Ok(t) => Outcome::Success(Self(t)),
      Err(e) => Outcome::Failure(e.into()),
    }
  }
}

impl<'a, T: FromJwt> OpenApiFromRequest<'a> for Auth<Jwt<T>> {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::Security(
      "jwt".to_owned(),
      SecurityScheme {
        description: None,
        data: SecuritySchemeData::Http {
          scheme: "bearer".to_owned(),
          bearer_format: Some("jwt".to_owned()),
        },
        extensions: Object::default(),
      },
      SecurityRequirement::default(),
    ))
  }
}

impl<'a> OpenApiFromRequest<'a> for Auth<OneTimeToken> {
  fn from_request_input(
    gen: &mut OpenApiGenerator,
    _name: String,
    required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    let schema = gen.json_schema::<OneTimeTokenParams>();

    Ok(RequestHeaderInput::Parameter(Parameter {
      name: "auth".to_owned(),
      location: "query".to_owned(),
      description: None,
      required,
      deprecated: false,
      allow_empty_value: false,
      value: ParameterValue::Schema {
        style: None,
        explode: None,
        allow_reserved: false,
        schema,
        example: None,
        examples: None,
      },
      extensions: Object::default(),
    }))
  }
}

impl<'a> OpenApiFromRequest<'a> for Auth<CoachInvitation> {
  fn from_request_input(
    gen: &mut OpenApiGenerator,
    _name: String,
    required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    let schema = gen.json_schema::<CoachInvitationParams>();

    Ok(RequestHeaderInput::Parameter(Parameter {
      name: "auth".to_owned(),
      location: "query".to_owned(),
      description: None,
      required,
      deprecated: false,
      allow_empty_value: false,
      value: ParameterValue::Schema {
        style: None,
        explode: None,
        allow_reserved: false,
        schema,
        example: None,
        examples: None,
      },
      extensions: Object::default(),
    }))
  }
}
