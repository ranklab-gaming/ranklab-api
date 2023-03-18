use crate::config::Config;
use crate::data_types::ReviewState;
use crate::models::{Coach, Recording};
use crate::schema::reviews;
use crate::stripe::TaxCalculation;
use derive_builder::Builder;
use diesel::dsl::{And, Eq, EqAny, Filter, FindBy, Or, Order};
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use stripe::{PaymentIntent, PaymentIntentId};
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Associations, Clone)]
#[diesel(belongs_to(Recording))]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "ReviewChangeset"
)]
#[builder_struct_attr(diesel(table_name = reviews))]
pub struct Review {
  pub coach_id: Uuid,
  pub created_at: chrono::NaiveDateTime,
  pub id: Uuid,
  pub notes: String,
  pub player_id: Uuid,
  pub recording_id: Uuid,
  pub state: ReviewState,
  pub stripe_payment_intent_id: String,
  pub updated_at: chrono::NaiveDateTime,
}

#[allow(clippy::type_complexity)]
impl Review {
  pub fn find_by_payment_intent_id<T: ToString>(
    order_id: &T,
  ) -> FindBy<reviews::table, reviews::stripe_payment_intent_id, Option<String>> {
    reviews::table.filter(reviews::stripe_payment_intent_id.eq(Some(order_id.to_string())))
  }

  pub fn find_for_player(
    id: &Uuid,
    player_id: &Uuid,
  ) -> Filter<reviews::table, And<Eq<reviews::id, Uuid>, Eq<reviews::player_id, Uuid>>> {
    reviews::table.filter(reviews::id.eq(*id).and(reviews::player_id.eq(*player_id)))
  }

  pub fn filter_for_player(
    player_id: &Uuid,
    archived: bool,
  ) -> Order<
    Filter<
      reviews::table,
      And<Eq<reviews::player_id, Uuid>, EqAny<reviews::state, Vec<ReviewState>>>,
    >,
    SqlLiteral<Bool>,
  > {
    let states = if archived {
      vec![ReviewState::Accepted, ReviewState::Refunded]
    } else {
      vec![
        ReviewState::Draft,
        ReviewState::AwaitingPayment,
        ReviewState::AwaitingReview,
        ReviewState::Published,
      ]
    };

    reviews::table
      .filter(
        reviews::player_id
          .eq(*player_id)
          .and(reviews::state.eq_any(states)),
      )
      .order(diesel::dsl::sql::<Bool>(
        "case \"state\"
          when 'awaiting_payment' then 1
          when 'published' then 2
          when 'draft' then 3
          when 'awaiting_review' then 4
        end,
        created_at desc",
      ))
  }

  pub fn filter_for_coach(
    coach: &Coach,
    archived: bool,
  ) -> Order<
    Filter<
      reviews::table,
      And<Eq<reviews::coach_id, Uuid>, EqAny<reviews::state, Vec<ReviewState>>>,
    >,
    SqlLiteral<Bool>,
  > {
    let states = if archived {
      vec![ReviewState::Accepted, ReviewState::Refunded]
    } else {
      vec![
        ReviewState::Draft,
        ReviewState::Published,
        ReviewState::AwaitingReview,
      ]
    };

    reviews::table
      .filter(
        reviews::coach_id
          .eq(coach.id)
          .and(reviews::state.eq_any(states)),
      )
      .order(diesel::dsl::sql::<Bool>(
        "case \"state\"
          when 'awaiting_review' then 3
          when 'draft' then 2
          when 'published' then 1
        end,
        created_at desc",
      ))
  }

  pub fn find_by_recording_for_coach(
    recording_id: &Uuid,
    coach_id: &Uuid,
  ) -> Filter<
    reviews::table,
    And<
      Or<Eq<reviews::coach_id, Uuid>, Eq<reviews::state, ReviewState>>,
      Eq<reviews::recording_id, Uuid>,
    >,
  > {
    reviews::table.filter(
      reviews::coach_id
        .eq(*coach_id)
        .or(reviews::state.eq(ReviewState::AwaitingReview))
        .and(reviews::recording_id.eq(*recording_id)),
    )
  }

  pub fn find_draft_for_coach(
    id: &Uuid,
    coach_id: &Uuid,
  ) -> Filter<
    reviews::table,
    And<And<Eq<reviews::coach_id, Uuid>, Eq<reviews::state, ReviewState>>, Eq<reviews::id, Uuid>>,
  > {
    reviews::table.filter(
      reviews::coach_id
        .eq(*coach_id)
        .and(reviews::state.eq(ReviewState::Draft))
        .and(reviews::id.eq(*id)),
    )
  }

  pub fn find_for_coach(
    id: &Uuid,
    coach_id: &Uuid,
  ) -> Filter<
    reviews::table,
    And<Or<Eq<reviews::coach_id, Uuid>, Eq<reviews::state, ReviewState>>, Eq<reviews::id, Uuid>>,
  > {
    reviews::table.filter(
      reviews::coach_id
        .eq(*coach_id)
        .or(reviews::state.eq(ReviewState::AwaitingReview))
        .and(reviews::id.eq(*id)),
    )
  }

  pub async fn get_payment_intent(&self, client: &stripe::Client) -> PaymentIntent {
    let payment_intent_id = self
      .stripe_payment_intent_id
      .parse::<PaymentIntentId>()
      .unwrap();

    stripe::PaymentIntent::retrieve(client, &payment_intent_id, &[])
      .await
      .unwrap()
  }
}
