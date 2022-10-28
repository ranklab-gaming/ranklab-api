use crate::config::Config;
use crate::emails::{Email, Recipient};
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
use serde_json::json;

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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ResetPasswordRequest {
  email: String,
  user_type: UserType,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ResetPasswordResponse {
  status: String,
}

enum Account {
  Coach(Coach),
  Player(Player),
}

fn generate_token(account: &Account, config: &Config) -> String {
  let now = Utc::now();
  let exp = (now + Duration::days(1)).timestamp() as usize;
  let sub = match account {
    Account::Coach(coach) => coach.id.to_string(),
    Account::Player(player) => player.id.to_string(),
  };
  let claims = Claims { sub, exp };
  let key = EncodingKey::from_secret(config.auth_client_secret.as_ref());
  encode(&Header::default(), &claims, &key).expect("failed to encode token")
}

#[openapi(tag = "Ranklab")]
#[post("/sessions", data = "<session>")]
pub async fn create(
  session: Json<CreateSessionRequest>,
  config: &State<Config>,
  db_conn: DbConn,
) -> MutationResponse<CreateSessionResponse> {
  let session_password = session.password.clone();

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

  let password = match &account {
    Account::Coach(coach) => coach.password.clone(),
    Account::Player(player) => player.password.clone(),
  };

  verify(session_password, &password)
    .map_err(|_| MutationError::Status(Status::UnprocessableEntity))?;

  let token = generate_token(&account, config);

  Response::success(CreateSessionResponse { token })
}

#[openapi(tag = "Ranklab")]
#[post("/sessions/reset-password", data = "<session>")]
pub async fn reset_password(
  session: Json<ResetPasswordRequest>,
  config: &State<Config>,
  db_conn: DbConn,
) -> MutationResponse<ResetPasswordResponse> {
  let email = session.email.clone();

  let response = Response::success(ResetPasswordResponse {
    status: "ok".into(),
  });

  let account = match session.user_type {
    UserType::Coach => Account::Coach(
      match db_conn
        .run(move |conn| Coach::find_by_email(&session.email).get_result::<Coach>(conn))
        .await
      {
        Ok(coach) => coach,
        Err(_) => return response,
      },
    ),
    UserType::Player => Account::Player(
      match db_conn
        .run(move |conn| Player::find_by_email(&session.email).get_result::<Player>(conn))
        .await
      {
        Ok(player) => player,
        Err(_) => return response,
      },
    ),
  };

  let name = match &account {
    Account::Coach(coach) => coach.name.clone(),
    Account::Player(player) => player.name.clone(),
  };

  let token = generate_token(&account, config);

  let email = Email::new(
    &config,
    "notification".to_owned(),
    json!({
      "subject": "Reset Your Password",
      "title": "Hello {{name}}, you requested to reset your password",
      "body": "Click the button below to reset it",
      "cta" : "Reset Password",
      "cta_url" : format!("{}/auth/reset-password?token={}", config.web_host, token),
    }),
    vec![Recipient::new(
      email,
      json!({
        "name": name,
      }),
    )],
  );

  email.deliver();

  response
}
