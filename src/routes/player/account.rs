use std::net::SocketAddr;

use crate::auth::{generate_token, Account};
use crate::config::Config;
use crate::games;
use crate::guards::{Auth, DbConn, Jwt, Stripe};
use crate::models::{Player, PlayerChangeset};
use crate::response::{MutationError, MutationResponse, QueryResponse, Response};
use crate::routes::session::CreateSessionResponse;
use crate::schema::players;
use crate::views::PlayerView;
use bcrypt::{hash, DEFAULT_COST};
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Deserialize, JsonSchema, Validate)]
pub struct UpdatePlayerRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(email)]
  email: String,
  #[validate(length(min = 1), custom = "crate::games::validate_id")]
  game_id: String,
  skill_level: i16,
  emails_enabled: bool,
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreatePlayerRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(email)]
  email: String,
  #[validate(length(min = 8))]
  password: String,
  #[validate(length(min = 1), custom = "crate::games::validate_id")]
  game_id: String,
  skill_level: i16,
}

#[openapi(tag = "Ranklab")]
#[get("/player/account")]
pub async fn get(auth: Auth<Jwt<Player>>, config: &State<Config>) -> QueryResponse<PlayerView> {
  Response::success(PlayerView::new(auth.into_deep_inner(), Some(config)))
}

#[openapi(tag = "Ranklab")]
#[post("/player/account", data = "<player>")]
pub async fn create(
  player: Json<CreatePlayerRequest>,
  db_conn: DbConn,
  stripe: Stripe,
  config: &State<Config>,
  ip_address: SocketAddr,
) -> MutationResponse<CreateSessionResponse> {
  if let Err(errors) = player.validate() {
    return Response::validation_error(errors);
  }

  let game = games::find(&player.game_id).unwrap();

  if !game
    .skill_levels
    .iter()
    .any(|skill_level| skill_level.value == player.skill_level as u8)
  {
    return Response::mutation_error(Status::UnprocessableEntity);
  }

  let mut params = stripe::CreateCustomer::new();

  params.email = Some(&player.email);

  params.tax = Some(stripe::CreateCustomerTax {
    ip_address: Some(ip_address.ip().to_string()),
  });

  let stripe = stripe
    .into_inner()
    .with_strategy(stripe::RequestStrategy::Idempotent(hex::encode(
      Sha256::digest(player.email.as_bytes()),
    )));

  let customer = stripe::Customer::create(&stripe, params).await.unwrap();

  let player = db_conn
    .run(move |conn| {
      diesel::insert_into(players::table)
        .values(
          PlayerChangeset::default()
            .password(hash(player.password.clone(), DEFAULT_COST).unwrap())
            .email(player.email.clone())
            .name(player.name.clone())
            .game_id(player.game_id.clone())
            .skill_level(player.skill_level)
            .stripe_customer_id(customer.id.to_string()),
        )
        .get_result::<Player>(conn)
    })
    .await
    .map_err(|err| match &err {
      diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
        if let Some(name) = info.constraint_name() {
          if name == "players_email_key" {
            let mut errors = ValidationErrors::new();
            errors.add("email", ValidationError::new("uniqueness"));
            return MutationError::ValidationErrors(errors);
          }
        };

        MutationError::InternalServerError(err.into())
      }
      _ => MutationError::InternalServerError(err.into()),
    })?;

  let account = Account::Player(player);
  let token = generate_token(&account, config);

  Response::success(CreateSessionResponse { token })
}

#[openapi(tag = "Ranklab")]
#[put("/player/account", data = "<account>")]
pub async fn update(
  account: Json<UpdatePlayerRequest>,
  auth: Auth<Jwt<Player>>,
  db_conn: DbConn,
  config: &State<Config>,
) -> MutationResponse<PlayerView> {
  if let Err(errors) = account.validate() {
    return Response::validation_error(errors);
  }

  let player = auth.into_deep_inner();
  let game = games::find(&player.game_id).unwrap();

  if !game
    .skill_levels
    .iter()
    .any(|skill_level| skill_level.value == player.skill_level as u8)
  {
    return Response::mutation_error(Status::UnprocessableEntity);
  }

  let player: Player = db_conn
    .run(move |conn| {
      diesel::update(&player)
        .set(
          PlayerChangeset::default()
            .email(account.email.clone())
            .name(account.name.clone())
            .game_id(account.game_id.clone())
            .skill_level(account.skill_level)
            .emails_enabled(account.emails_enabled),
        )
        .get_result::<Player>(conn)
    })
    .await
    .map_err(|err| match &err {
      diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
        if let Some(name) = info.constraint_name() {
          if name == "players_email_key" {
            let mut errors = ValidationErrors::new();
            errors.add("email", ValidationError::new("uniqueness"));
            return MutationError::ValidationErrors(errors);
          }
        };

        MutationError::InternalServerError(err.into())
      }
      _ => MutationError::InternalServerError(err.into()),
    })?;

  Response::success(PlayerView::new(player, Some(config)))
}
