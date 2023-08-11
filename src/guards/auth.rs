use okapi::openapi3::*;
use rocket::request::{FromRequest, Outcome, Request};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
mod jwt;
mod ott;
use self::ott::ToScope;
pub use self::ott::{Ott, ResetPassword};
pub use jwt::Jwt;
pub use ott::OneTimeTokenParams;
use rocket::http::Status;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
  #[error("missing authorization header")]
  Missing,
  #[error("invalid token: {0}")]
  Invalid(String),
  #[error("user not found")]
  NotFound,
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
      Ok(t) => Outcome::Success(Auth(t)),
      Err(e) => match e {
        AuthError::Missing => Outcome::Failure((Status::Unauthorized, e)),
        AuthError::Invalid(_) => Outcome::Failure((Status::BadRequest, e)),
        AuthError::NotFound => Outcome::Failure((Status::NotFound, e)),
      },
    }
  }
}

#[async_trait]
impl<'r, T: AuthFromRequest> FromRequest<'r> for Auth<Option<T>> {
  type Error = AuthError;

  async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    match T::from_request(request).await {
      Ok(t) => Outcome::Success(Auth(Some(t))),
      Err(e) => match e {
        AuthError::Missing => Outcome::Success(Auth(None)),
        AuthError::Invalid(_) => Outcome::Failure((Status::BadRequest, e)),
        AuthError::NotFound => Outcome::Failure((Status::NotFound, e)),
      },
    }
  }
}

impl<'a> OpenApiFromRequest<'a> for Auth<Jwt> {
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

impl<'a> OpenApiFromRequest<'a> for Auth<Option<Jwt>> {
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

impl<'a, T: ToScope> OpenApiFromRequest<'a> for Auth<Ott<T>> {
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
