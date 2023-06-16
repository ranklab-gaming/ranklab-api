use super::{Auth, AuthError};
use crate::guards::auth::AuthFromRequest;
use crate::guards::DbConn;
use crate::models::OneTimeToken;
use diesel::prelude::*;
use rocket::Request;
use schemars::JsonSchema;
use serde::Deserialize;

pub struct ResetPassword;
pub struct Ott<T>(OneTimeToken, T);

pub trait ToScope {
  fn new() -> Self;
  fn to_scope() -> String;
}

impl ToScope for ResetPassword {
  fn new() -> Self {
    Self
  }

  fn to_scope() -> String {
    "reset_password".to_string()
  }
}

impl<T> Auth<Ott<T>> {
  pub fn into_token(self) -> OneTimeToken {
    self.into_inner().0
  }
}

#[derive(Debug, Deserialize, JsonSchema, FromForm)]
pub struct OneTimeTokenParams {
  token: String,
}

#[async_trait]
impl<T: ToScope> AuthFromRequest for Ott<T> {
  async fn from_request<'r>(req: &'r Request<'_>) -> Result<Self, AuthError> {
    let db_conn = req.guard::<DbConn>().await.unwrap();

    let query = match req.query_value::<OneTimeTokenParams>("auth") {
      Some(Ok(query)) => query,
      _ => return Err(AuthError::Missing),
    };

    let token = db_conn
      .run(move |conn| {
        OneTimeToken::find_by_value(&query.token, &T::to_scope()).first::<OneTimeToken>(conn)
      })
      .await
      .map_err(|_| AuthError::NotFound)?;

    Ok(Ott(token, T::new()))
  }
}
