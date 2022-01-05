use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Coach, Comment};
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use uuid::Uuid;

#[derive(FromForm, JsonSchema)]
pub struct ListCommentsQuery {
  review_id: Uuid,
}

#[openapi(tag = "Ranklab")]
#[get("/player/comments?<params..>")]
pub async fn list(
  params: ListCommentsQuery,
  _auth: Auth<Coach>,
  db_conn: DbConn,
) -> Json<Vec<Comment>> {
  let comments = db_conn
    .run(move |conn| {
      use crate::schema::comments::dsl::*;
      comments
        .filter(review_id.eq(params.review_id))
        .load(conn)
        .unwrap()
    })
    .await;

  Json(comments)
}
