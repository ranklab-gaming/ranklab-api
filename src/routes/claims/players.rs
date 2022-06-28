use std::net::SocketAddr;

use crate::data_types::UserGame;
use crate::guards::auth::Claims;
use crate::guards::{Auth, DbConn, Stripe};
use crate::models::{Player, PlayerChangeset};
use crate::response::{MutationResponse, Response};
use crate::schema::players;
use crate::views::PlayerView;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreatePlayerRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(length(min = 1))]
  games: Vec<UserGame>,
}

#[openapi(tag = "Ranklab")]
#[post("/claims/players", data = "<player>")]
pub async fn create(
  player: Json<CreatePlayerRequest>,
  auth: Auth<Claims>,
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
            .email(auth.0.email.clone())
            .name(player.name.clone())
            .auth0_id(auth.0.sub.clone())
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
