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

#[derive(FromForm, JsonSchema)]
pub struct ListReviewsQuery {
  page: Option<i64>,
  archived: Option<bool>,
}

#[openapi(tag = "Ranklab")]
#[get("/player/reviews?<params..>")]
pub async fn list(
  params: ListReviewsQuery,
  auth: Auth<Player>,
  db_conn: DbConn,
) -> QueryResponse<PaginatedResult<ReviewView>> {
  let paginated_reviews: PaginatedResult<Review> = db_conn
    .run(move |conn| {
      Review::filter_for_player(&auth.0.id, params.archived.unwrap_or(false))
        .paginate(params.page.unwrap_or(1))
        .load_and_count_pages::<Review>(conn)
        .unwrap()
    })
    .await;

  let review_views = paginated_reviews
    .records
    .clone()
    .into_iter()
    .map(|review| ReviewView::from(review, None))
    .collect();

  Response::success(paginated_reviews.records(review_views))
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
    .run(move |conn| Review::find_for_player(&id, &auth.0.id).first::<Review>(conn))
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
      diesel::insert_into(reviews::table)
        .values(
          ReviewChangeset::default()
            .recording_id(body_recording_id)
            .player_id(auth_player_id)
            .title(body.title.clone())
            .notes(ammonia::clean(&body.notes))
            .game_id(body.game_id.clone()),
        )
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

  let mut price_data = CreateOrderLineItemsPriceData {
    currency: Some(stripe::Currency::USD),
    product: Some(product_id.to_string()),
    ..Default::default()
  };
  price_data.unit_amount = Some(10_00);

  let mut line_item = CreateOrderLineItems::default();
  line_item.quantity = Some(1);
  line_item.price_data = Some(price_data);

  let line_items = vec![line_item];

  let mut payment_settings = CreateOrderPaymentSettings::default();
  payment_settings.payment_method_types =
    Some(vec![CreateOrderPaymentSettingsPaymentMethodTypes::Card]);
  payment_settings.payment_method_options =
    Some(stripe::CreateOrderPaymentSettingsPaymentMethodOptions {
      card: Some(stripe::CreateOrderPaymentSettingsPaymentMethodOptionsCard {
        capture_method: None,
        setup_future_usage: Some(
          stripe::CreateOrderPaymentSettingsPaymentMethodOptionsCardSetupFutureUsage::OnSession,
        ),
      }),
      ..Default::default()
    });

  let mut params = CreateOrder::new(stripe::Currency::USD, line_items);

  params.customer = Some(customer_id);
  params.description = Some("Recording payment");
  params.payment = Some(CreateOrderPayment {
    settings: payment_settings,
  });
  params.ip_address = Some(&ip_address);
  // TODO: enable when we add a valid address in test mode
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

  stripe::PaymentIntent::update(&stripe.0 .0, &payment_intent_id, payment_intent_params)
    .await
    .unwrap();

  let order_id = order.id.clone();

  let review = db_conn
    .run(move |conn| {
      diesel::update(&review)
        .set(ReviewChangeset::default().stripe_order_id(order_id.to_string()))
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
