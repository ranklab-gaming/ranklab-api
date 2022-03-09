use crate::data_types::ReviewState;
use crate::schema::reviews;
use uuid::Uuid;

#[derive(Queryable, Identifiable)]
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
