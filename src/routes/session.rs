use crate::auth::{generate_token, Account, UserType};
use crate::config::Config;
use crate::emails::{Email, Recipient};
use crate::guards::{Auth, DbConn};
use crate::models::{
  Coach, CoachChangeset, OneTimeToken, OneTimeTokenChangeset, Player, PlayerChangeset,
};
use crate::response::{MutationError, MutationResponse, Response, StatusResponse};
use crate::schema::one_time_tokens;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use diesel::prelude::*;
use rand::distributions::{Alphanumeric, DistString};
use rocket::figment::Provider;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
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

  let valid = verify(session_password, &password)
    .map_err(|_| MutationError::Status(Status::UnprocessableEntity))?;

  if !valid {
    return Response::mutation_error(Status::NotFound);
  }

  let token = generate_token(&account, config);

  Response::success(CreateSessionResponse { token })
}

#[openapi(tag = "Ranklab")]
#[post("/sessions/reset-password", data = "<reset_password>")]
pub async fn reset_password(
  reset_password: Json<ResetPasswordRequest>,
  config: &State<Config>,
  db_conn: DbConn,
  rocket_config: &rocket::Config,
) -> MutationResponse<StatusResponse> {
  let profile = rocket_config.profile().unwrap();
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

  let user_type = match &account {
    Account::Coach(_) => "coach",
    Account::Player(_) => "player",
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

  let reset_password_email = Email::new(
    config,
    "notification".to_owned(),
    json!({
      "subject": "Reset Your Password",
      "title": "You requested to reset your password",
      "body": "Click the button below to reset it",
      "cta" : "Reset Password",
      "cta_url" : format!("{}/password/reset?token={}&user_type={}", config.web_host, token.value, user_type),
    }),
    vec![Recipient::new(
      email,
      json!({
        "name": name,
      }),
    )],
  );

  if profile != "test" {
    reset_password_email.deliver().await.unwrap();
  }

  response
}

#[openapi(tag = "Ranklab")]
#[put("/sessions/password", data = "<password>")]
pub async fn update_password(
  password: Json<UpdatePasswordRequest>,
  db_conn: DbConn,
  auth: Auth<OneTimeToken>,
) -> MutationResponse<StatusResponse> {
  let token = auth.into_inner();
  let account = token.account(&db_conn).await?;
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

  db_conn
    .run(move |conn| {
      diesel::update(&token)
        .set(OneTimeTokenChangeset::default().used_at(Some(Utc::now().naive_utc())))
        .get_result::<OneTimeToken>(conn)
        .unwrap()
    })
    .await;

  Response::status(Status::Ok)
}
