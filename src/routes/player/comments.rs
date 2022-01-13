use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Coach, Comment, Review};
use crate::response;
use crate::response::QueryResponse;
use diesel::prelude::*;
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
  auth: Auth<Coach>,
  db_conn: DbConn,
) -> QueryResponse<Vec<Comment>> {
  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews
        .filter(player_id.eq(auth.0.id).and(id.eq(params.review_id)))
        .first::<Review>(conn)
    })
    .await?;

  let comments = db_conn
    .run(move |conn| {
      use crate::schema::comments::dsl::*;
      comments.filter(review_id.eq(review.id)).load(conn).unwrap()
    })
    .await;

  response::success(comments)
}
