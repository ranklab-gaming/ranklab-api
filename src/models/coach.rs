use crate::data_types::UserGame;
use uuid::Uuid;

#[derive(Queryable)]
pub struct Coach {
  pub id: Uuid,
  pub name: String,
  pub email: String,
  pub bio: String,
  pub games: Vec<UserGame>,
  pub auth0_id: String,
  pub stripe_account_id: Option<String>,
  pub stripe_details_submitted: bool,
  pub stripe_payouts_enabled: bool,
  pub country: String,
}
