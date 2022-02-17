use crate::data_types::UserGame;
use crate::schema::coaches;
use uuid::Uuid;

#[derive(Queryable, Identifiable)]
#[table_name = "coaches"]
pub struct Coach {
  pub auth0_id: String,
  pub bio: String,
  pub country: String,
  pub email: String,
  pub games: Vec<UserGame>,
  pub id: Uuid,
  pub name: String,
  pub stripe_account_id: Option<String>,
  pub stripe_details_submitted: bool,
  pub stripe_payouts_enabled: bool,
}
