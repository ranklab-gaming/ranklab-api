use super::AuthError;
use crate::auth::UserType;
use crate::guards::auth::AuthFromRequest;
use crate::guards::DbConn;
use crate::models::{CoachInvitation, OneTimeToken};
use diesel::prelude::*;
use rocket::Request;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema, FromForm)]
pub struct OneTimeTokenParams {
  token: String,
  user_type: UserType,
}

#[derive(Debug, Deserialize, JsonSchema, FromForm)]
pub struct CoachInvitationParams {
  token: String,
}

#[async_trait]
impl AuthFromRequest for OneTimeToken {
  async fn from_request<'r>(req: &'r Request<'_>) -> Result<Self, AuthError> {
    let db_conn = req.guard::<DbConn>().await.unwrap();

    let query = match req.query_value::<OneTimeTokenParams>("auth") {
      Some(Ok(query)) => query,
      _ => return Err(AuthError::Missing),
    };

    let token = db_conn
      .run(move |conn| {
        OneTimeToken::find_by_value(&query.token, query.user_type).first::<OneTimeToken>(conn)
      })
      .await
      .map_err(|_| AuthError::NotFound("token".to_string()))?;

    Ok(token)
  }
}

#[async_trait]
impl AuthFromRequest for CoachInvitation {
  async fn from_request<'r>(req: &'r Request<'_>) -> Result<Self, AuthError> {
    let db_conn = req.guard::<DbConn>().await.unwrap();

    let query = match req.query_value::<CoachInvitationParams>("auth") {
      Some(Ok(query)) => query,
      _ => return Err(AuthError::Missing),
    };

    let invitation = db_conn
      .run(move |conn| CoachInvitation::find_by_value(&query.token).first::<CoachInvitation>(conn))
      .await
      .map_err(|_| AuthError::NotFound("token".to_string()))?;

    Ok(invitation)
  }
}
