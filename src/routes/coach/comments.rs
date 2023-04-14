use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Coach, Comment, CommentChangeset, Review};
use crate::response::{MutationResponse, QueryResponse, Response, StatusResponse};
use crate::schema::comments;
use crate::views::CommentView;
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
  video_timestamp: Option<i32>,
  review_id: Uuid,
  drawing: String,
  metadata: Option<serde_json::Value>,
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
  auth: Auth<Jwt<Coach>>,
  db_conn: DbConn,
) -> MutationResponse<CommentView> {
  if let Err(errors) = comment.validate() {
    return Response::validation_error(errors);
  }

  let review_id = comment.review_id;
  let coach_id = auth.into_deep_inner().id;

  let review: Review = db_conn
    .run(move |conn| Review::find_draft_for_coach(&review_id, &coach_id).first(conn))
    .await?;

  let comment: CommentView = db_conn
    .run(move |conn| {
      diesel::insert_into(comments::table)
        .values(
          CommentChangeset::default()
            .body(ammonia::clean(&comment.body))
            .video_timestamp(comment.video_timestamp)
            .review_id(review.id)
            .coach_id(coach_id)
            .drawing(comment.drawing.clone())
            .metadata(comment.metadata.clone()),
        )
        .get_result::<Comment>(conn)
        .unwrap()
    })
    .await
    .into();

  Response::success(comment)
}

#[openapi(tag = "Ranklab")]
#[put("/coach/comments/<id>", data = "<comment>")]
pub async fn update(
  id: Uuid,
  comment: Json<UpdateCommentRequest>,
  auth: Auth<Jwt<Coach>>,
  db_conn: DbConn,
) -> MutationResponse<CommentView> {
  if let Err(errors) = comment.validate() {
    return Response::validation_error(errors);
  }

  let coach_id = auth.into_deep_inner().id;

  let existing_comment = db_conn
    .run(move |conn| Comment::find_for_coach(&id, &coach_id).first::<Comment>(conn))
    .await?;

  let updated_comment: CommentView = db_conn
    .run(move |conn| {
      diesel::update(&existing_comment)
        .set(
          CommentChangeset::default()
            .body(comment.body.clone())
            .drawing(comment.drawing.clone()),
        )
        .get_result::<Comment>(conn)
        .unwrap()
    })
    .await
    .into();

  Response::success(updated_comment)
}

#[openapi(tag = "Ranklab")]
#[delete("/coach/comments/<id>")]
pub async fn delete(
  id: Uuid,
  auth: Auth<Jwt<Coach>>,
  db_conn: DbConn,
) -> MutationResponse<StatusResponse> {
  let coach_id = auth.into_deep_inner().id;

  let existing_comment = db_conn
    .run(move |conn| Comment::find_for_coach(&id, &coach_id).first::<Comment>(conn))
    .await?;

  db_conn
    .run(move |conn| diesel::delete(&existing_comment).execute(conn))
    .await?;

  Response::status(Status::NoContent)
}

#[derive(FromForm, JsonSchema)]
pub struct ListCommentsQuery {
  review_id: Uuid,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/comments?<params..>")]
pub async fn list(
  params: ListCommentsQuery,
  auth: Auth<Jwt<Coach>>,
  db_conn: DbConn,
) -> QueryResponse<Vec<CommentView>> {
  let comments: Vec<CommentView> = db_conn
    .run(move |conn| {
      Comment::filter_by_review_for_coach(&params.review_id, &auth.into_deep_inner().id)
        .load::<Comment>(conn)
        .unwrap()
    })
    .await
    .into_iter()
    .map(Into::into)
    .collect();

  Response::success(comments)
}
