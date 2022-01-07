use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Coach, Comment, Review};
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
#[post("/coach/comments", data = "<comment>")]
pub async fn create(
  comment: Json<CreateCommentRequest>,
  auth: Auth<Coach>,
  db_conn: DbConn,
) -> Response<Comment> {
  let review_id = comment.review_id.clone();
  let auth_id = auth.0.id.clone();

  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;

      reviews
        .filter(id.eq(review_id).and(coach_id.eq(Some(auth_id.clone()))))
        .first::<Review>(conn)
    })
    .await?;

  if let Err(errors) = comment.validate() {
    return Response::ValidationErrors(errors);
  }

  let comment = db_conn
    .run(move |conn| {
      use crate::schema::comments::dsl::*;

      diesel::insert_into(comments)
        .values((
          body.eq(comment.body.clone()),
          video_timestamp.eq(comment.video_timestamp),
          review_id.eq(review.id),
          coach_id.eq(auth_id),
          drawing.eq(comment.drawing.clone()),
        ))
        .get_result(conn)
        .unwrap()
    })
    .await;

  Response::Success(comment)
}

#[openapi(tag = "Ranklab")]
#[put("/coach/comments/<comment_id>", data = "<comment>")]
pub async fn update(
  comment_id: Uuid,
  comment: Json<UpdateCommentRequest>,
  auth: Auth<Coach>,
  db_conn: DbConn,
) -> Response<Comment> {
  let auth_id = auth.0.id.clone();

  let existing_comment = db_conn
    .run(move |conn| {
      use crate::schema::comments::dsl::*;

      comments
        .filter(id.eq(comment_id).and(coach_id.eq(auth_id)))
        .first::<Comment>(conn)
    })
    .await;

  match existing_comment {
    Err(diesel::result::Error::NotFound) => Response::Status(Status::NotFound),
    Ok(existing_comment) => {
      if let Err(errors) = comment.validate() {
        return Response::ValidationErrors(errors);
      }

      let updated_comment = db_conn
        .run(move |conn| {
          use crate::schema::comments::dsl::*;

          diesel::update(crate::schema::comments::table.find(existing_comment.id))
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
    Err(error) => panic!("Error: {}", error),
  }
}

#[derive(FromForm, JsonSchema)]
pub struct ListCommentsQuery {
  review_id: Uuid,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/comments?<params..>")]
pub async fn list(
  params: ListCommentsQuery,
  auth: Auth<Coach>,
  db_conn: DbConn,
) -> Json<Vec<Comment>> {
  let comments = db_conn
    .run(move |conn| {
      use crate::schema::comments::dsl::*;

      comments
        .filter(review_id.eq(params.review_id).and(coach_id.eq(auth.0.id)))
        .load(conn)
        .unwrap()
    })
    .await;

  Json(comments)
}
