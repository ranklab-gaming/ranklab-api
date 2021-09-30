use crate::db::DbConn;
use crate::guards::auth::ApiKey;
use crate::guards::Auth;
use crate::models::User;
use crate::response::Response;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct CreateUserRequest {
  auth0_id: String,
}

#[openapi]
#[post("/users", data = "<user>")]
pub async fn create(
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
#[get("/users/current")]
pub async fn get_current(auth: Auth<User>) -> Json<User> {
  Json(auth.0)
}
