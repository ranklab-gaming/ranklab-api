use std::collections::HashMap;
use std::net::SocketAddr;

use crate::auth::{decode_token_credentials, generate_token, Account, Credentials};
use crate::config::Config;
use crate::emails::{Email, Recipient};
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
use rocket::figment::Provider;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Deserialize, JsonSchema, Validate)]
pub struct UpdatePlayerRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(length(min = 1), custom = "crate::games::validate_id")]
  game_id: String,
  skill_level: i16,
  emails_enabled: bool,
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreatePlayerRequest {
  #[validate(length(min = 2))]
  name: String,
  credentials: Credentials,
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
  rocket_config: &rocket::Config,
) -> MutationResponse<CreateSessionResponse> {
  let profile = rocket_config.profile().unwrap();

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

  let email = match &player.credentials {
    Credentials::Password(credentials) => credentials.email.clone(),
    Credentials::Token(credentials) => decode_token_credentials(&credentials, config)
      .ok_or_else(|| MutationError::Status(Status::UnprocessableEntity))?
      .sub
      .clone(),
  };

  let password = match &player.credentials {
    Credentials::Password(credentials) => Some(credentials.password.clone()),
    Credentials::Token(_) => None,
  };

  params.email = Some(&email);

  params.tax = Some(stripe::CreateCustomerTax {
    ip_address: Some(ip_address.ip().to_string()),
  });

  let mut metadata = HashMap::new();

  if let Some(instance_id) = config.instance_id.as_ref() {
    metadata.insert("instance_id".to_owned(), instance_id.to_owned());
  }

  params.metadata = Some(metadata);

  let stripe = stripe
    .into_inner()
    .with_strategy(stripe::RequestStrategy::Idempotent(hex::encode(
      Sha256::digest(email.as_bytes()),
    )));

  let customer = stripe::Customer::create(&stripe, params).await.unwrap();

  let player = db_conn
    .run(move |conn| {
      diesel::insert_into(players::table)
        .values(
          PlayerChangeset::default()
            .password(password.map(|password| hash(password.clone(), DEFAULT_COST).unwrap()))
            .email(email.clone())
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

  let name = player.name.clone();
  let email = player.email.clone();
  let account = Account::Player(player);
  let token = generate_token(&account, config);

  let player_signup_email = Email::new(
    config,
    "notification".to_owned(),
    json!({
      "subject": "A player has signed up!",
      "title": format!("{} has signed up to Ranklab", name),
      "body": format!("Their email is: {}", email),
    }),
    vec![Recipient::new(
      "sales@ranklab.gg".to_owned(),
      json!({
        "name": "Ranklab",
      }),
    )],
  );

  if profile == rocket::config::Config::RELEASE_PROFILE {
    player_signup_email.deliver().await.unwrap();
  }

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
