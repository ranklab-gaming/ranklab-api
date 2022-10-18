use std::net::SocketAddr;

use crate::data_types::PlayerGame;
use crate::guards::{Auth, DbConn, Stripe};
use crate::models::{Player, PlayerChangeset};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::schema::players;
use crate::views::PlayerView;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, JsonSchema)]
#[schemars(rename = "PlayerUpdateAccountRequest")]
pub struct UpdateAccountRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(email)]
  email: String,
  #[validate(length(min = 1))]
  games: Vec<PlayerGame>,
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreatePlayerRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(email)]
  email: String,
  #[validate(length(min = 1))]
  games: Vec<PlayerGame>,
}

#[openapi(tag = "Ranklab")]
#[get("/player/account")]
pub async fn get(auth: Auth<Player>) -> QueryResponse<PlayerView> {
  Response::success(auth.0.into())
}

#[openapi(tag = "Ranklab")]
#[post("/player/account", data = "<player>")]
pub async fn create(
  player: Json<CreatePlayerRequest>,
  db_conn: DbConn,
  stripe: Stripe,
  ip_address: SocketAddr,
) -> MutationResponse<PlayerView> {
  if let Err(errors) = player.validate() {
    return Response::validation_error(errors);
  }

  let player: Player = db_conn
    .run(move |conn| {
      diesel::insert_into(players::table)
        .values(
          PlayerChangeset::default()
            .email(player.email.clone())
            .name(player.name.clone())
            .games(player.games.clone().into_iter().map(|g| Some(g)).collect())
            .stripe_customer_id(None),
        )
        .get_result(conn)
        .unwrap()
    })
    .await;

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

  let customer = stripe::Customer::create(&stripe.0 .0, params)
    .await
    .unwrap();

  let player: PlayerView = db_conn
    .run(move |conn| {
      diesel::update(&player)
        .set(PlayerChangeset::default().stripe_customer_id(Some(customer.id.to_string())))
        .get_result::<Player>(conn)
        .unwrap()
    })
    .await
    .into();

  Response::success(player)
}

#[openapi(tag = "Ranklab")]
#[put("/player/account", data = "<account>")]
pub async fn update(
  account: Json<UpdateAccountRequest>,
  auth: Auth<Player>,
  db_conn: DbConn,
) -> MutationResponse<PlayerView> {
  let player = auth.0.clone();

  let player: PlayerView = db_conn
    .run(move |conn| {
      diesel::update(&player)
        .set(
          PlayerChangeset::default()
            .email(account.email.clone())
            .name(account.name.clone())
            .games(account.games.clone().into_iter().map(|g| Some(g)).collect()),
        )
        .get_result::<Player>(conn)
        .unwrap()
    })
    .await
    .into();

  Response::success(player)
}
