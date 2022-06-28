use std::collections::HashMap;
use std::net::SocketAddr;

use crate::config::Config;
use crate::data_types::ReviewState;
use crate::guards::{Auth, DbConn, Stripe};
use crate::models::{Coach, Player, Review, ReviewChangeset};
use crate::pagination::{Paginate, PaginatedResult};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::schema::{coaches, reviews};
use crate::views::ReviewView;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;
use stripe::{
  CreateOrder, CreateOrderLineItems, CreateOrderLineItemsPriceData, CreateOrderPayment,
  CreateOrderPaymentSettings, CreateOrderPaymentSettingsPaymentMethodTypes, Expandable, Order,
  OrderId, SubmitOrder,
};
use uuid::Uuid;

#[derive(Deserialize, JsonSchema)]
#[schemars(rename = "PlayerUpdateAccountRequest")]
pub struct UpdateAccountRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(email)]
  email: String,
  #[validate(length(min = 1))]
  games: Vec<UserGame>,
}

#[openapi(tag = "Ranklab")]
#[put("/player/account", data = "<account>")]
pub async fn update(
  id: Uuid,
  account: Json<UpdateAccountRequest>,
  auth: Auth<Player>,
  db_conn: DbConn,
) -> MutationResponse<ReviewView> {
  let auth_id = auth.0.id.clone();

  let existing_review: Review = db_conn
    .run(move |conn| Review::find_for_player(&id, &auth_id).first(conn))
    .await?;

  if !review.accepted {
    return Response::success(ReviewView::from(existing_review, None));
  }

  let updated_review = db_conn
    .run(move |conn| {
      diesel::update(&existing_review)
        .set(ReviewChangeset::default().state(ReviewState::Accepted))
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  let review_coach_id = updated_review.coach_id.unwrap().clone();

  let coach: Coach = db_conn
    .run(move |conn| coaches::table.find(&review_coach_id).first(conn).unwrap())
    .await;

  let stripe_order_id = updated_review.stripe_order_id.parse::<OrderId>().unwrap();

  let order = Order::retrieve(&stripe.0 .0, &stripe_order_id, &["payment.payment_intent"])
    .await
    .unwrap();

  let payment_intent = match order.payment.payment_intent {
    Some(Expandable::Object(payment_intent)) => payment_intent,
    _ => panic!("No payment intent found"),
  };

  let mut transfer_params =
    stripe::CreateTransfer::new(stripe::Currency::USD, coach.stripe_account_id.unwrap());
  transfer_params.amount = Some((order.amount_total as f64 * 0.8) as i64);
  transfer_params.source_transaction = Some(payment_intent.charges.data[0].id.clone());

  stripe::Transfer::create(&stripe.0 .0, transfer_params)
    .await
    .unwrap();

  Response::success(ReviewView::from(updated_review, None))
}
