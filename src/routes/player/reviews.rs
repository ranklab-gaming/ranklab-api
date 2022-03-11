use std::collections::HashMap;
use std::net::SocketAddr;

use crate::config::Config;
use crate::data_types::ReviewState;
use crate::guards::{Auth, DbConn, Stripe};
use crate::models::{Coach, Player, Review};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::stripe::order::{
  CreateOrder, CreateOrderLineItem, CreateOrderLineItemPriceData, CreateOrderPayment, Order,
  OrderId, OrderPaymentSettings, OrderPaymentSettingsPaymentMethodType, SubmitOrder,
};
use crate::views::ReviewView;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;
use stripe::Expandable;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/player/reviews")]
pub async fn list(auth: Auth<Player>, db_conn: DbConn) -> QueryResponse<Vec<ReviewView>> {
  let reviews: Vec<ReviewView> = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews
        .filter(player_id.eq(auth.0.id))
        .load::<Review>(conn)
        .unwrap()
    })
    .await
    .into_iter()
    .map(|review| ReviewView::from(review, None))
    .collect();

  Response::success(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/player/reviews/<id>")]
pub async fn get(
  id: Uuid,
  auth: Auth<Player>,
  db_conn: DbConn,
  stripe: Stripe,
) -> QueryResponse<ReviewView> {
  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{id as review_id, player_id, reviews};
      reviews
        .filter(player_id.eq(auth.0.id).and(review_id.eq(id)))
        .first::<Review>(conn)
    })
    .await?;

  let stripe_order_id = review.stripe_order_id.parse::<OrderId>().unwrap();

  let order = Order::retrieve(&stripe.0 .0, &stripe_order_id, &["payment.payment_intent"])
    .await
    .unwrap();

  let payment_intent = match order.payment.payment_intent.clone() {
    Some(Expandable::Object(payment_intent)) => payment_intent,
    _ => panic!("No payment intent found"),
  };

  Response::success(ReviewView::from(review, Some(*payment_intent)))
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateReviewMutation {
  recording_id: Uuid,
  #[validate(length(min = 1))]
  title: String,
  notes: String,
  #[validate(custom = "crate::games::validate_id")]
  game_id: String,
}

#[openapi(tag = "Ranklab")]
#[post("/player/reviews", data = "<body>")]
pub async fn create(
  db_conn: DbConn,
  auth: Auth<Player>,
  stripe: Stripe,
  body: Json<CreateReviewMutation>,
  config: &State<Config>,
  ip_address: SocketAddr,
) -> MutationResponse<ReviewView> {
  let body_recording_id = body.recording_id.clone();
  let auth_player_id = auth.0.id.clone();

  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;

      diesel::insert_into(reviews)
        .values((
          recording_id.eq(body_recording_id),
          player_id.eq(auth_player_id),
          title.eq(body.title.clone()),
          notes.eq(body.notes.clone()),
          game_id.eq(body.game_id.clone()),
        ))
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  let customer_id = auth
    .0
    .stripe_customer_id
    .unwrap()
    .parse::<stripe::CustomerId>()
    .unwrap();

  let product_id = config
    .stripe_product_id
    .parse::<stripe::ProductId>()
    .unwrap();

  let ip_address = match ip_address.ip() {
    std::net::IpAddr::V4(ip) => ip.to_string(),
    std::net::IpAddr::V6(ip) => ip.to_ipv4().unwrap().to_string(),
  };

  let mut price_data = CreateOrderLineItemPriceData::new(stripe::Currency::USD, product_id.clone());
  price_data.unit_amount = Some(10_00);

  let mut line_item = CreateOrderLineItem::new();
  line_item.quantity = Some(1);
  line_item.price_data = Some(price_data);

  let line_items = vec![line_item];

  let mut payment_settings = OrderPaymentSettings::new();
  payment_settings.payment_method_types = Some(vec![OrderPaymentSettingsPaymentMethodType::Card]);

  let mut params = CreateOrder::new(stripe::Currency::USD, line_items);

  params.customer = Some(customer_id);
  params.description = Some("Recording payment".to_string());
  params.payment = Some(CreateOrderPayment {
    settings: payment_settings,
  });
  params.ip_address = Some(ip_address);
  // enable when we add a valid address in test mode
  // params.automatic_tax = Some(CreateOrderAutomaticTax { enabled: true });

  let submit_params = SubmitOrder {
    expected_total: 10_00,
    expand: &["payment.payment_intent"],
  };

  let order = Order::create(&stripe.0 .0, params).await.unwrap();
  let order = Order::submit(&stripe.0 .0, &order.id, submit_params)
    .await
    .unwrap();

  let payment_intent_id = match order.payment.payment_intent.clone() {
    Some(Expandable::Id(payment_intent_id)) => payment_intent_id,
    Some(Expandable::Object(payment_intent)) => payment_intent.id,
    None => panic!("No payment intent found"),
  };

  let mut payment_intent_params = stripe::UpdatePaymentIntent::new();
  let mut payment_intent_metadata = HashMap::new();

  payment_intent_metadata.insert("order_id".to_string(), order.id.to_string());
  payment_intent_params.metadata = Some(payment_intent_metadata);

  // TODO: Ask support if this is possible
  // payment_intent_params.setup_future_usage =
  //   Some(stripe::PaymentIntentSetupFutureUsageFilter::OnSession);

  stripe::PaymentIntent::update(&stripe.0 .0, &payment_intent_id, payment_intent_params)
    .await
    .unwrap();

  let order_id = order.id.clone();

  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;

      diesel::update(&review)
        .set(stripe_order_id.eq(order_id.to_string()))
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  Response::success(ReviewView::from(review, None))
}

#[derive(Deserialize, JsonSchema)]
#[schemars(rename = "PlayerUpdateReviewRequest")]
pub struct UpdateReviewRequest {
  accepted: bool,
}

#[openapi(tag = "Ranklab")]
#[put("/player/reviews/<id>", data = "<review>")]
pub async fn update(
  id: Uuid,
  review: Json<UpdateReviewRequest>,
  auth: Auth<Player>,
  db_conn: DbConn,
  stripe: Stripe,
) -> MutationResponse<ReviewView> {
  let auth_id = auth.0.id.clone();

  let existing_review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{id as review_id, player_id, reviews, state};

      reviews
        .filter(
          review_id
            .eq(id)
            .and(state.eq(ReviewState::Published).or(player_id.eq(auth_id))),
        )
        .first::<Review>(conn)
    })
    .await?;

  if !review.accepted {
    return Response::success(ReviewView::from(existing_review, None));
  }

  let updated_review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::state;

      diesel::update(&existing_review)
        .set(state.eq(ReviewState::Accepted))
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  let review_coach_id = updated_review.coach_id.unwrap().clone();

  let coach = db_conn
    .run(move |conn| {
      use crate::schema::coaches::dsl::{coaches, id as coach_id};

      coaches
        .filter(coach_id.eq(review_coach_id))
        .first::<Coach>(conn)
        .unwrap()
    })
    .await;

  let stripe_order_id = updated_review.stripe_order_id.parse::<OrderId>().unwrap();

  let order = Order::retrieve(&stripe.0 .0, &stripe_order_id, &[])
    .await
    .unwrap();

  let mut transfer_params =
    stripe::CreateTransfer::new(stripe::Currency::USD, coach.stripe_account_id.unwrap());
  transfer_params.amount = Some((order.amount_total as f64 * 0.8) as i64);

  stripe::Transfer::create(&stripe.0 .0, transfer_params)
    .await
    .unwrap();

  Response::success(ReviewView::from(updated_review, None))
}
