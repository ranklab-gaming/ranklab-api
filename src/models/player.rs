use crate::data_types::UserGame;
use uuid::Uuid;

#[derive(Queryable)]
pub struct Player {
  pub id: Uuid,
  pub auth0_id: String,
  pub name: String,
  pub email: String,
  pub games: Vec<UserGame>,
  pub stripe_customer_id: Option<String>
}
