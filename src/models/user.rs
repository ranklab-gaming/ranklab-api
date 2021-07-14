use rocket::serde::Serialize;

#[derive(Queryable, Serialize)]
pub struct User {
  pub id: String,
  pub auth0_id: String,
}
