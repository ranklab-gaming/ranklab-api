use crate::config::Config;
use crate::emails::{Email, Recipient};
use crate::guards::auth::UserType;
use crate::guards::{Auth, DbConn};
use crate::models::{
  Account, Coach, CoachChangeset, OneTimeToken, OneTimeTokenChangeset, Player, PlayerChangeset,
};
use crate::response::{MutationError, MutationResponse, Response, StatusResponse};
use crate::schema::one_time_tokens;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::prelude::*;
use chrono::Duration;
use diesel::prelude::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::distributions::{Alphanumeric, DistString};
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
  iss: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CreateSessionResponse {
  pub token: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ResetPasswordRequest {
  email: String,
  user_type: UserType,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdatePasswordRequest {
  password: String,
}

pub fn generate_token(account: &Account, config: &Config) -> String {
  let now = Utc::now();
  let exp = (now + Duration::minutes(1)).timestamp() as usize;

  let sub = match account {
    Account::Coach(coach) => format!("coach:{}", coach.id.to_string()),
    Account::Player(player) => format!("player:{}", player.id.to_string()),
  };

  let claims = Claims {
    sub,
    exp,
    iss: config.host.clone(),
  };

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
#[post("/sessions/reset-password", data = "<reset_password>")]
pub async fn reset_password(
  reset_password: Json<ResetPasswordRequest>,
  config: &State<Config>,
  db_conn: DbConn,
) -> MutationResponse<StatusResponse> {
  let email = reset_password.email.clone();
  let response = Response::status(Status::Ok);

  let account = match reset_password.user_type {
    UserType::Coach => Account::Coach(
      match db_conn
        .run(move |conn| Coach::find_by_email(&reset_password.email).get_result::<Coach>(conn))
        .await
      {
        Ok(coach) => coach,
        Err(_) => return response,
      },
    ),
    UserType::Player => Account::Player(
      match db_conn
        .run(move |conn| Player::find_by_email(&reset_password.email).get_result::<Player>(conn))
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

  let token: OneTimeToken = db_conn
    .run(move |conn| {
      let value = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

      diesel::insert_into(one_time_tokens::table)
        .values(
          OneTimeTokenChangeset::default()
            .value(value)
            .scope("reset-password".to_owned())
            .player_id(match &account {
              Account::Coach(_) => None,
              Account::Player(player) => Some(player.id),
            })
            .coach_id(match &account {
              Account::Coach(coach) => Some(coach.id),
              Account::Player(_) => None,
            }),
        )
        .get_result::<OneTimeToken>(conn)
        .unwrap()
    })
    .await;

  let email = Email::new(
    &config,
    "notification".to_owned(),
    json!({
      "subject": "Reset Your Password",
      "title": "Hello {{name}}, you requested to reset your password",
      "body": "Click the button below to reset it",
      "cta" : "Reset Password",
      "cta_url" : format!("{}/auth/reset-password?token={}", config.web_host, token.value),
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

#[openapi(tag = "Ranklab")]
#[put("/sessions/password", data = "<password>")]
pub async fn update_password(
  password: Json<UpdatePasswordRequest>,
  db_conn: DbConn,
  auth: Auth<OneTimeToken>,
) -> MutationResponse<StatusResponse> {
  let account = auth.0.account(&db_conn).await?;
  let password_hash = hash(&password.password, DEFAULT_COST).unwrap();

  match account {
    Account::Coach(coach) => {
      db_conn
        .run(move |conn| {
          diesel::update(&coach)
            .set(CoachChangeset::default().password(password_hash))
            .get_result::<Coach>(conn)
            .unwrap()
        })
        .await;
    }
    Account::Player(player) => {
      db_conn
        .run(move |conn| {
          diesel::update(&player)
            .set(PlayerChangeset::default().password(password_hash))
            .get_result::<Player>(conn)
            .unwrap()
        })
        .await;
    }
  }

  Response::status(Status::Ok)
}
