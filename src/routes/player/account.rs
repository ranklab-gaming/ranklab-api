use std::net::SocketAddr;

use crate::config::Config;
use crate::data_types::PlayerGame;
use crate::guards::{Auth, DbConn, Stripe};
use crate::models::{Account, Player, PlayerChangeset};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::routes::session::{generate_token, CreateSessionResponse};
use crate::schema::players;
use crate::views::PlayerView;
use bcrypt::{hash, DEFAULT_COST};
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::State;
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
  #[validate(length(min = 8))]
  password: String,
  #[validate(length(min = 1))]
  games: Vec<PlayerGame>,
}

#[openapi(tag = "Ranklab")]
#[get("/player/account")]
pub async fn get(auth: Auth<Player>) -> QueryResponse<PlayerView> {
  Response::success(auth.0.into())
}

#[openapi(tag = "Ranklab")]
#[post("/player/account", data = "<request>")]
pub async fn create(
  request: Json<CreatePlayerRequest>,
  db_conn: DbConn,
  stripe: Stripe,
  ip_address: SocketAddr,
  config: &State<Config>,
) -> MutationResponse<CreateSessionResponse> {
  if let Err(errors) = request.validate() {
    return Response::validation_error(errors);
  }

  let player: Player = db_conn
    .run(move |conn| {
      diesel::insert_into(players::table)
        .values(
          PlayerChangeset::default()
            .password(
              hash(request.password.clone(), DEFAULT_COST).expect("Failed to hash password"),
            )
            .email(request.email.clone())
            .name(request.name.clone())
            .games(request.games.clone().into_iter().map(|g| Some(g)).collect())
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

  let player = db_conn
    .run(move |conn| {
      diesel::update(&player)
        .set(PlayerChangeset::default().stripe_customer_id(Some(customer.id.to_string())))
        .get_result::<Player>(conn)
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
