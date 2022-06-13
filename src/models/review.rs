use crate::data_types::ReviewState;
use crate::models::{Coach, Recording};
use crate::schema::reviews;
use derive_builder::Builder;
use diesel::dsl::{sql, And, Eq, Filter, FindBy, Or};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable};
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Recording))]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "ReviewChangeset"
)]
#[builder_struct_attr(diesel(table_name = reviews))]
pub struct Review {
  pub coach_id: Option<Uuid>,
  pub game_id: String,
  pub id: Uuid,
  pub notes: String,
  pub player_id: Uuid,
  pub recording_id: Uuid,
  pub skill_level: i16,
  pub title: String,
  pub state: ReviewState,
  pub stripe_order_id: String,
  pub updated_at: chrono::NaiveDateTime,
  pub created_at: chrono::NaiveDateTime,
}

type NullableBoxedExpression =
  Box<dyn BoxableExpression<reviews::table, Pg, SqlType = Nullable<Bool>>>;

type BoxedExpression = Box<dyn BoxableExpression<reviews::table, Pg, SqlType = Bool>>;

impl Review {
  pub fn find_by_order_id<T: ToString>(
    order_id: &T,
  ) -> FindBy<reviews::table, reviews::stripe_order_id, String> {
    reviews::table.filter(reviews::stripe_order_id.eq(order_id.to_string()))
  }

  pub fn find_for_player(
    id: &Uuid,
    player_id: &Uuid,
  ) -> Filter<reviews::table, And<Eq<reviews::id, Uuid>, Eq<reviews::player_id, Uuid>>> {
    reviews::table.filter(reviews::id.eq(*id).and(reviews::player_id.eq(*player_id)))
  }

  pub fn filter_for_player(
    player_id: &Uuid,
  ) -> Filter<reviews::table, Eq<reviews::player_id, Uuid>> {
    reviews::table.filter(reviews::player_id.eq(*player_id))
  }

  pub fn filter_for_coach(coach: &Coach, archived: bool) -> reviews::BoxedQuery<'_, Pg> {
    let mut games_expression: BoxedExpression = Box::new(sql("false"));

    for game_option in coach.games.clone().into_iter() {
      let game = game_option.unwrap().clone();

      games_expression = Box::new(
        games_expression.or(
          reviews::game_id
            .eq(game.game_id)
            .and(reviews::skill_level.lt(game.skill_level as i16)),
        ),
      );
    }

    let mut filter_expression: NullableBoxedExpression = Box::new(reviews::coach_id.eq(coach.id));

    filter_expression = if archived {
      Box::new(
        filter_expression
          .and(reviews::state.eq_any(vec![ReviewState::Accepted, ReviewState::Refunded])),
      )
    } else {
      Box::new(
        filter_expression
          .and(reviews::state.eq_any(vec![ReviewState::Draft, ReviewState::Published]))
          .or(
            reviews::state
              .eq(ReviewState::AwaitingReview)
              .and(reviews::coach_id.is_null()),
          ),
      )
    };

    reviews::table
      .filter(filter_expression.and(games_expression))
      .order(diesel::dsl::sql::<Bool>(
        "case \"state\"
          when 'awaiting_review' then 3
          when 'draft' then 2
          when 'published' then 1
        end,
        created_at desc",
      ))
      .into_boxed()
  }

  pub fn find_by_recording_for_coach(
    recording_id: &Uuid,
    coach_id: &Uuid,
  ) -> Filter<
    reviews::table,
    And<
      Or<Eq<reviews::coach_id, Option<Uuid>>, Eq<reviews::state, ReviewState>>,
      Eq<reviews::recording_id, Uuid>,
    >,
  > {
    reviews::table.filter(
      reviews::coach_id
        .eq(Some(*coach_id))
        .or(reviews::state.eq(ReviewState::AwaitingReview))
        .and(reviews::recording_id.eq(*recording_id)),
    )
  }

  pub fn find_draft_for_coach(
    id: &Uuid,
    coach_id: &Uuid,
  ) -> Filter<
    reviews::table,
    And<
      And<Eq<reviews::coach_id, Option<Uuid>>, Eq<reviews::state, ReviewState>>,
      Eq<reviews::id, Uuid>,
    >,
  > {
    reviews::table.filter(
      reviews::coach_id
        .eq(Some(*coach_id))
        .and(reviews::state.eq(ReviewState::Draft))
        .and(reviews::id.eq(*id)),
    )
  }

  pub fn find_for_coach(
    id: &Uuid,
    coach_id: &Uuid,
  ) -> Filter<
    reviews::table,
    And<
      Or<Eq<reviews::coach_id, Option<Uuid>>, Eq<reviews::state, ReviewState>>,
      Eq<reviews::id, Uuid>,
    >,
  > {
    reviews::table.filter(
      reviews::coach_id
        .eq(Some(*coach_id))
        .or(reviews::state.eq(ReviewState::AwaitingReview))
        .and(reviews::id.eq(*id)),
    )
  }
}
