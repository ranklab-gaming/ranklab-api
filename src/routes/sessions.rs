use crate::auth::{
  decode_token_credentials, generate_token, Credentials, PasswordCredentials, TokenCredentials,
};
use crate::config::Config;
use crate::guards::DbConn;
use crate::models::{Session, User};
use crate::response::{MutationError, MutationResponse, Response};
use bcrypt::verify;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct CreateSessionRequest {
  credentials: Credentials,
}

async fn create_with_password(
  credentials: &PasswordCredentials,
  config: &State<Config>,
  db_conn: DbConn,
) -> Option<String> {
  let session_password = credentials.password.clone();
  let email = credentials.email.clone();

  let user = db_conn
    .run(move |conn| User::find_by_email(&email).get_result::<User>(conn))
    .await
    .ok()?;

  let password = user.password.clone();

  match password {
    Some(password) => {
      let valid = verify(session_password, &password).ok()?;

      if !valid {
        return None;
      }
    }
    None => return None,
  }

  Some(generate_token(&user, config))
}

async fn create_with_token(
  credentials: &TokenCredentials,
  config: &State<Config>,
  db_conn: DbConn,
) -> Option<String> {
  let claims = decode_token_credentials(credentials, config)?;
  let email = claims.sub;

  let user = db_conn
    .run(move |conn| User::find_by_email(&email).get_result::<User>(conn))
    .await
    .ok()?;

  Some(generate_token(&user, config))
}

#[openapi(tag = "Ranklab")]
#[post("/sessions", data = "<session>")]
pub async fn create(
  session: Json<CreateSessionRequest>,
  config: &State<Config>,
  db_conn: DbConn,
) -> MutationResponse<Session> {
  let token = match &session.credentials {
    Credentials::Password(password) => create_with_password(password, config, db_conn).await,
    Credentials::Token(token) => create_with_token(token, config, db_conn).await,
  }
  .ok_or(MutationError::Status(Status::NotFound))?;

  Response::success(Session { token })
}
