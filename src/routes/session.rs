use crate::config::Config;
use crate::guards::auth::UserType;
use crate::guards::DbConn;
use crate::models::{Coach, Player};
use crate::response::{MutationError, MutationResponse, Response};
use bcrypt::verify;
use chrono::prelude::*;
use chrono::Duration;
use diesel::prelude::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use rocket::serde::json::Json;
use rocket::State;
use rocket_http::Status;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
pub struct CreateSessionRequest {
  email: String,
  password: String,
  user_type: UserType,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String,
  exp: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CreateSessionResponse {
  token: String,
}

enum Account {
  Coach(Coach),
  Player(Player),
}

#[openapi(tag = "Ranklab")]
#[post("/sessions", data = "<session>")]
pub async fn create(
  session: Json<CreateSessionRequest>,
  config: &State<Config>,
  db_conn: DbConn,
) -> MutationResponse<CreateSessionResponse> {
  let session_password = session.password.clone();
  let session_email = session.email.clone();

  let account = match session.user_type {
    UserType::Coach => Account::Coach(
      db_conn
        .run(move |conn| Coach::find_by_email(&session.email).get_result::<Coach>(conn))
        .await?,
    ),
    UserType::Player => Account::Player(
      db_conn
        .run(move |conn| Player::find_by_email(&session.email).get_result::<Player>(conn))
        .await?,
    ),
  };

  let password = match account {
    Account::Coach(coach) => coach.password,
    Account::Player(player) => player.password,
  };

  verify(session_password, &password)
    .map_err(|_| MutationError::Status(Status::UnprocessableEntity))?;

  let claims = Claims {
    sub: session_email,
    exp: (Utc::now() + Duration::days(1)).timestamp() as usize,
  };

  let token = encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(config.auth_client_secret.as_ref()),
  )
  .expect("failed to encode token");

  Response::success(CreateSessionResponse { token })
}
