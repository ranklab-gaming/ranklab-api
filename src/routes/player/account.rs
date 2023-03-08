use crate::auth::{generate_token, Account};
use crate::config::Config;
use crate::games;
use crate::guards::{Auth, DbConn, Jwt, Stripe};
use crate::models::{Player, PlayerChangeset};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::routes::session::CreateSessionResponse;
use crate::schema::players;
use crate::views::PlayerView;
use bcrypt::{hash, DEFAULT_COST};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;
use std::net::SocketAddr;
use validator::Validate;

#[derive(Deserialize, JsonSchema, Validate)]
pub struct UpdatePlayerRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(email)]
  email: String,
  #[validate(length(min = 1), custom = "crate::games::validate_id")]
  game_id: String,
  skill_level: i16,
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
pub async fn get(auth: Auth<Jwt<Player>>) -> QueryResponse<PlayerView> {
  Response::success(auth.into_deep_inner().into())
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

  if game
    .skill_levels
    .iter()
    .find(|skill_level| skill_level.value == player.skill_level as u8)
    .is_none()
  {
    return Response::mutation_error(Status::UnprocessableEntity);
  }

  let ip_address = match ip_address.ip() {
    std::net::IpAddr::V4(ip) => ip.to_string(),
    std::net::IpAddr::V6(ip) => ip.to_ipv4().unwrap().to_string(),
  };

  let mut params = stripe::CreateCustomer::new();

  params.email = Some(&player.email);
  params.tax = Some(
    stripe::CreateCustomerTax {
      ip_address: Some(ip_address.into()),
    }
    .into(),
  );

  let customer = stripe::Customer::create(&stripe.into_inner(), params)
    .await
    .unwrap();

  let player: Player = db_conn
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
        .get_result(conn)
        .unwrap()
    })
    .await;

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
) -> MutationResponse<PlayerView> {
  if let Err(errors) = account.validate() {
    return Response::validation_error(errors);
  }

  let player = auth.into_deep_inner();
  let game = games::find(&player.game_id).unwrap();

  if game
    .skill_levels
    .iter()
    .find(|skill_level| skill_level.value == player.skill_level as u8)
    .is_none()
  {
    return Response::mutation_error(Status::UnprocessableEntity);
  }

  let player: PlayerView = db_conn
    .run(move |conn| {
      diesel::update(&player)
        .set(
          PlayerChangeset::default()
            .email(account.email.clone())
            .name(account.name.clone())
            .game_id(account.game_id.clone())
            .skill_level(account.skill_level),
        )
        .get_result::<Player>(conn)
        .unwrap()
    })
    .await
    .into();

  Response::success(player)
}
