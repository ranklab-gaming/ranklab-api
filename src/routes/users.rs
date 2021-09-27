use crate::db::DbConn;
use crate::guards::auth::ApiKey;
use crate::guards::Auth;
use crate::models::User;
use crate::response::Response;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::Route;
use rocket_okapi::{openapi, openapi_get_routes as routes};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct CreateUserRequest {
  auth0_id: String,
}

#[openapi]
#[post("/", data = "<user>")]
async fn create_user(
  user: Json<CreateUserRequest>,
  db_conn: DbConn,
  _auth: Auth<ApiKey>,
) -> Response<User> {
  let user = db_conn
    .run(move |conn| {
      use crate::schema::users::dsl::*;

      diesel::insert_into(users)
        .values(&vec![(auth0_id.eq(user.auth0_id.clone()))])
        .get_result(conn)
        .unwrap()
    })
    .await;

  Response::Success(user)
}

#[openapi]
#[get("/current")]
async fn get_current_user(auth: Auth<User>) -> Json<User> {
  Json(auth.0)
}

pub fn build() -> Vec<Route> {
  routes![create_user, get_current_user]
}
