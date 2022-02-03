use crate::data_types::UserGame;
use crate::guards::auth::Claims;
use crate::guards::Auth;
use crate::guards::DbConn;
use crate::guards::Stripe;
use crate::models::Player;
use crate::response::{MutationResponse, Response};
use crate::views::PlayerView;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreatePlayerRequest {
  #[validate(length(min = 1))]
  name: String,
  games: Vec<UserGame>,
}

#[openapi(tag = "Ranklab")]
#[post("/claims/players", data = "<player>")]
pub async fn create(
  player: Json<CreatePlayerRequest>,
  auth: Auth<Claims>,
  db_conn: DbConn,
  stripe: Stripe,
) -> MutationResponse<PlayerView> {
  if let Err(errors) = player.validate() {
    return Response::validation_error(errors);
  }

  let player: Player = db_conn
    .run(move |conn| {
      use crate::schema::players::dsl::*;

      diesel::insert_into(players)
        .values((
          email.eq(auth.0.email.clone()),
          name.eq(player.name.clone()),
          auth0_id.eq(auth.0.sub.clone()),
          games.eq(player.games.clone()),
          stripe_customer_id.eq::<Option<String>>(None),
        ))
        .get_result(conn)
        .unwrap()
    })
    .await;

  let mut params = stripe::CreateCustomer::new();
  params.email = Some(&player.email);

  let customer = stripe::Customer::create(&stripe.0, params).await.unwrap();

  let player: PlayerView = db_conn
    .run(move |conn| {
      use crate::schema::players::dsl::*;

      diesel::update(crate::schema::players::table.find(player.id))
        .set(stripe_customer_id.eq(Some(customer.id.to_string())))
        .get_result::<Player>(conn)
        .unwrap()
    })
    .await
    .into();

  Response::success(player)
}
