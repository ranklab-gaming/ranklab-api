use crate::data_types::UserGame;
use crate::schema::players;
use uuid::Uuid;

#[derive(Queryable, Identifiable)]
pub struct Player {
  pub auth0_id: String,
  pub email: String,
  pub games: Vec<UserGame>,
  pub id: Uuid,
  pub name: String,
  pub stripe_customer_id: Option<String>,
}
