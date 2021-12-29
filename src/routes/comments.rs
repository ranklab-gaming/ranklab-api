use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Comment, Player, Review};
use crate::response::Response;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateCommentRequest {
  #[validate(length(min = 1))]
  body: String,
  video_timestamp: i32,
  review_id: Uuid,
  drawing: String,
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct UpdateCommentRequest {
  #[validate(length(min = 1))]
  body: String,
  drawing: String,
}

#[openapi(tag = "Ranklab")]
#[post("/comments", data = "<comment>")]
pub async fn create(
  comment: Json<CreateCommentRequest>,
  auth: Auth<Player>,
  db_conn: DbConn,
) -> Response<Comment> {
  if let Err(errors) = comment.validate() {
    return Response::ValidationErrors(errors);
  }

  let review_id = comment.review_id;

  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews.find(review_id).first::<Review>(conn)
    })
    .await;

  if let Err(diesel::result::Error::NotFound) = review {
    return Response::Status(Status::UnprocessableEntity);
  }

  let comment = db_conn
    .run(move |conn| {
      use crate::schema::comments::dsl::*;

      diesel::insert_into(comments)
        .values((
          body.eq(comment.body.clone()),
          video_timestamp.eq(comment.video_timestamp),
          review_id.eq(review.unwrap().id),
          coach_id.eq(auth.0.id.clone()),
          drawing.eq(comment.drawing.clone()),
        ))
        .get_result(conn)
        .unwrap()
    })
    .await;

  Response::Success(comment)
}

#[openapi(tag = "Ranklab")]
#[put("/comments/<id>", data = "<comment>")]
pub async fn update(
  id: Uuid,
  comment: Json<UpdateCommentRequest>,
  _auth: Auth<Player>,
  db_conn: DbConn,
) -> Response<Comment> {
  if let Err(errors) = comment.validate() {
    return Response::ValidationErrors(errors);
  }

  let existing_comment = crate::schema::comments::table.find(id);

  let updated_comment = db_conn
    .run(move |conn| {
      use crate::schema::comments::dsl::*;

      diesel::update(existing_comment)
        .set((
          body.eq(comment.body.clone()),
          drawing.eq(comment.drawing.clone()),
        ))
        .get_result(conn)
        .unwrap()
    })
    .await;

  Response::Success(updated_comment)
}

#[derive(FromForm, JsonSchema)]
pub struct ListCommentsQuery {
  review_id: Uuid,
}

#[openapi(tag = "Ranklab")]
#[get("/comments?<params..>")]
pub async fn list(
  params: ListCommentsQuery,
  _auth: Auth<Player>,
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
