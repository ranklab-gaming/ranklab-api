use crate::config::Config;
use crate::emails::{Email, Recipient};
use crate::guards::auth::{Ott, ResetPassword};
use crate::guards::{Auth, DbConn};
use crate::models::{OneTimeToken, OneTimeTokenChangeset, User, UserChangeset};
use crate::response::{MutationResponse, Response, StatusResponse};
use crate::schema::one_time_tokens;
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use diesel::prelude::*;
use rand::distributions::{Alphanumeric, DistString};
use rocket::figment::Provider;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreatePasswordRequest {
  email: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdatePasswordRequest {
  password: String,
}

#[openapi(tag = "Ranklab")]
#[post("/passwords", data = "<password>")]
pub async fn create(
  password: Json<CreatePasswordRequest>,
  config: &State<Config>,
  db_conn: DbConn,
  rocket_config: &rocket::Config,
) -> MutationResponse<StatusResponse> {
  let profile = rocket_config.profile().unwrap();
  let email = password.email.clone();
  let response = Response::status(Status::Ok);

  let user = match db_conn
    .run(move |conn| User::find_by_email(&password.email).get_result::<User>(conn))
    .await
  {
    Ok(user) => user,
    Err(_) => return response,
  };

  let name = user.name.clone();

  let token: OneTimeToken = db_conn
    .run(move |conn| {
      let value = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

      diesel::insert_into(one_time_tokens::table)
        .values(
          OneTimeTokenChangeset::default()
            .value(value)
            .scope("reset_password".to_owned())
            .user_id(Some(user.id)),
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
      "cta_url" : format!("{}/password/reset?token={}", config.web_host, token.value),
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
#[put("/passwords", data = "<password>")]
pub async fn update(
  password: Json<UpdatePasswordRequest>,
  db_conn: DbConn,
  auth: Auth<Ott<ResetPassword>>,
) -> MutationResponse<StatusResponse> {
  let token = auth.into_token();
  let user_id = token.user_id.unwrap();

  let user = db_conn
    .run(move |conn| User::find_by_id(&user_id).get_result::<User>(conn).unwrap())
    .await;

  let password_hash = hash(&password.password, DEFAULT_COST).unwrap();

  db_conn
    .run(move |conn| {
      diesel::update(&user)
        .set(UserChangeset::default().password(Some(password_hash)))
        .get_result::<User>(conn)
        .unwrap()
    })
    .await;

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
