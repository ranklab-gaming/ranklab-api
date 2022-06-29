//! Error type for auth0 requests.
use reqwest::header::ToStrError;
use serde_json::Error as JsonError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;

use crate::clients::auth0_management::token::TokenError;
use serde::Deserialize;

#[derive(Debug)]
pub enum RateLimitError {
  MissingRateLimitHeader,
  MissingRateResetHeader,
  MissingRateRemainingHeader,
  BadHeaderEncoding(ToStrError),
  BadHeaderFormat(ParseIntError),
}

impl From<ToStrError> for RateLimitError {
  fn from(err: ToStrError) -> Self {
    RateLimitError::BadHeaderEncoding(err)
  }
}

impl From<ParseIntError> for RateLimitError {
  fn from(err: ParseIntError) -> Self {
    RateLimitError::BadHeaderFormat(err)
  }
}

impl Display for RateLimitError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Error for RateLimitError {}

/// The error returned when querying Auth0.
#[derive(Debug)]
pub enum Auth0Error {
  /// Json error
  Json(JsonError),
  /// Generic http error.
  Http(reqwest::Error),
  /// Authentication token error.
  Token(TokenError),
  /// Auth0 server side error.
  Auth0(String),
  /// Auth0 rate limit error.
  RateLimit(RateLimitError),
}

impl Display for Auth0Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Error for Auth0Error {}

impl From<JsonError> for Auth0Error {
  fn from(err: JsonError) -> Self {
    Auth0Error::Json(err)
  }
}

impl From<TokenError> for Auth0Error {
  fn from(inner: TokenError) -> Self {
    Auth0Error::Token(inner)
  }
}

impl From<reqwest::Error> for Auth0Error {
  fn from(inner: reqwest::Error) -> Self {
    Auth0Error::Http(inner)
  }
}

impl From<RateLimitError> for Auth0Error {
  fn from(inner: RateLimitError) -> Self {
    Auth0Error::RateLimit(inner)
  }
}

/// Auth0 error response.
#[derive(Deserialize)]
pub struct Auth0ErrorResponse {
  message: Option<String>,
}

impl From<Auth0ErrorResponse> for Auth0Error {
  fn from(inner: Auth0ErrorResponse) -> Self {
    Auth0Error::Auth0(inner.message.unwrap_or_else(|| "".to_owned()))
  }
}
