use rocket::serde::Serialize;

#[derive(Serialize)]
pub struct Recording {
  pub id: String,
  pub upload_url: String,
}
