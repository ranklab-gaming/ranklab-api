use crate::data_types::ReviewState;
use crate::models::Coach;
use crate::schema::reviews;
use derive_builder::Builder;
use diesel::dsl::{sql, And, Eq, Filter, FindBy, Or};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "ReviewChangeset"
)]
#[builder_struct_attr(table_name = "reviews")]
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
}

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

  pub fn filter_for_coach(coach: &Coach, pending: Option<bool>) -> reviews::BoxedQuery<'_, Pg> {
    let mut games_expression: Box<dyn BoxableExpression<reviews::table, Pg, SqlType = Bool>> =
      Box::new(sql::<Bool>("false"));

    for game in coach.games.clone().into_iter() {
      games_expression = Box::new(
        games_expression.or(
          reviews::game_id
            .eq(game.game_id)
            .and(reviews::skill_level.lt(game.skill_level as i16)),
        ),
      );
    }

    if pending.unwrap_or(false) {
      reviews::table
        .filter(
          reviews::state
            .eq(ReviewState::AwaitingReview)
            .and(games_expression),
        )
        .into_boxed()
    } else {
      reviews::table
        .filter(reviews::coach_id.eq(coach.id))
        .into_boxed()
    }
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
