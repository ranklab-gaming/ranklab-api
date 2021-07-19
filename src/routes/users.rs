use crate::db::DbConn;
use crate::guards::auth::ApiKey;
use crate::guards::Auth;
use crate::models::User;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::Route;

#[derive(Deserialize)]
struct CreateUserRequest {
  auth0_id: String,
}

#[post("/", data = "<user>")]
async fn create_user(
  user: Json<CreateUserRequest>,
  db_conn: DbConn,
  _auth: Auth<ApiKey>,
) -> Json<User> {
  use crate::schema::users::dsl::*;

  let user = db_conn
    .run(move |conn| {
      diesel::insert_into(users)
        .values(&vec![(auth0_id.eq(user.auth0_id.clone()))])
        .get_result(conn)
        .unwrap()
    })
    .await;

  Json(user)
}

#[get("/")]
async fn current_user(auth: Auth<User>) -> Json<User> {
  Json(auth.0)
}

pub fn build() -> Vec<Route> {
  routes![create_user, current_user]
}
