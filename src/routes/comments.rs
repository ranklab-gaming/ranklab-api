use std::collections::HashSet;

use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Comment, CommentChangeset, CommentMetadata, User};
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
  body: String,
  recording_id: Uuid,
  metadata: CommentMetadata,
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct UpdateCommentRequest {
  body: String,
  metadata: CommentMetadata,
}

#[derive(FromForm, JsonSchema)]
pub struct ListParams {
  recording_id: Uuid,
}

#[openapi(tag = "Ranklab")]
#[post("/comments", data = "<comment>")]
pub async fn create(
  comment: Json<CreateCommentRequest>,
  auth: Auth<Jwt>,
  db_conn: DbConn,
) -> MutationResponse<CommentView> {
  if let Err(errors) = comment.validate() {
    return Response::validation_error(errors);
  }

  let recording_id = comment.recording_id;
  let user = auth.into_user();
  let user_id = user.id;

  let comment = db_conn
    .run(move |conn| {
      let mut metadata_cleaner = ammonia::Builder::default();
      let mut allowed_tags = HashSet::new();

      allowed_tags.insert("svg");
      allowed_tags.insert("path");

      metadata_cleaner
        .tags(allowed_tags)
        .add_tag_attributes("svg", &["viewBox"])
        .add_tag_attributes(
          "path",
          &[
            "stroke",
            "fill",
            "stroke-linecap",
            "stroke-linejoin",
            "stroke-width",
            "d",
          ],
        );

      let metadata = match &comment.metadata {
        CommentMetadata::Video { timestamp, drawing } => CommentMetadata::Video {
          timestamp: *timestamp,
          drawing: metadata_cleaner.clean(drawing).to_string(),
        },
      };

      diesel::insert_into(comments::table)
        .values(
          CommentChangeset::default()
            .body(ammonia::clean(&comment.body))
            .recording_id(recording_id)
            .user_id(user_id)
            .metadata(serde_json::to_value(metadata).unwrap()),
        )
        .get_result::<Comment>(conn)
        .unwrap()
    })
    .await;

  Response::success(CommentView::new(comment, Some(user)))
}

#[openapi(tag = "Ranklab")]
#[put("/comments/<id>", data = "<comment>")]
pub async fn update(
  id: Uuid,
  comment: Json<UpdateCommentRequest>,
  auth: Auth<Jwt>,
  db_conn: DbConn,
) -> MutationResponse<CommentView> {
  if let Err(errors) = comment.validate() {
    return Response::validation_error(errors);
  }

  let user = auth.into_user();
  let user_id = user.id;

  let existing_comment = db_conn
    .run(move |conn| Comment::find_for_user(&user_id, &id).first::<Comment>(conn))
    .await?;

  let updated_comment = db_conn
    .run(move |conn| {
      diesel::update(&existing_comment)
        .set(
          CommentChangeset::default()
            .body(comment.body.clone())
            .metadata(serde_json::to_value(&comment.metadata).unwrap()),
        )
        .get_result::<Comment>(conn)
        .unwrap()
    })
    .await;

  Response::success(CommentView::new(updated_comment, Some(user)))
}

#[openapi(tag = "Ranklab")]
#[delete("/comments/<id>")]
pub async fn delete(
  id: Uuid,
  auth: Auth<Jwt>,
  db_conn: DbConn,
) -> MutationResponse<StatusResponse> {
  let user_id = auth.into_user().id;

  let existing_comment = db_conn
    .run(move |conn| Comment::find_for_user(&user_id, &id).first::<Comment>(conn))
    .await?;

  db_conn
    .run(move |conn| diesel::delete(&existing_comment).execute(conn))
    .await?;

  Response::status(Status::NoContent)
}

#[openapi(tag = "Ranklab")]
#[get("/comments?<params..>")]
pub async fn list(params: ListParams, db_conn: DbConn) -> QueryResponse<Vec<CommentView>> {
  let recording_id = params.recording_id;

  let comments = db_conn
    .run(move |conn| {
      Comment::filter_by_recording_id(&recording_id)
        .load::<Comment>(conn)
        .unwrap()
    })
    .await;

  let user_ids = comments
    .iter()
    .map(|comment| comment.user_id)
    .collect::<HashSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();

  let users = db_conn
    .run(move |conn| User::filter_by_ids(user_ids).load::<User>(conn).unwrap())
    .await;

  let comments = comments
    .into_iter()
    .map(|comment| {
      let user = users
        .iter()
        .find(|user| user.id == comment.user_id)
        .cloned();

      CommentView::new(comment, user)
    })
    .collect();

  Response::success(comments)
}
