use crate::schema::review_intents;
use uuid::Uuid;

#[derive(Queryable, Identifiable)]
pub struct ReviewIntent {
  pub game_id: String,
  pub id: Uuid,
  pub notes: String,
  pub player_id: Uuid,
  pub recording_id: Option<Uuid>,
  pub review_id: Option<Uuid>,
  pub stripe_payment_intent_id: String,
  pub title: String,
}
