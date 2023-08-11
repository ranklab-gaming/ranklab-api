use std::collections::HashSet;

use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Audio, Comment, CommentChangeset, CommentMetadata, User};
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
  audio_id: Option<Uuid>,
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct UpdateCommentRequest {
  body: String,
  metadata: CommentMetadata,
  audio_id: Option<Uuid>,
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
  let mut audio: Option<Audio> = None;

  if let Some(audio_id) = comment.audio_id {
    audio = Some(
      db_conn
        .run(move |conn| Audio::find_for_user(&user_id, &audio_id).first::<Audio>(conn))
        .await?,
    );
  }

  let audio_id = audio.as_ref().map(|audio| audio.id);

  let comment: Comment = db_conn
    .run(move |conn| {
      diesel::insert_into(comments::table)
        .values(
          CommentChangeset::default()
            .body(ammonia::clean(&comment.body))
            .recording_id(recording_id)
            .user_id(user_id)
            .metadata(serde_json::to_value(&comment.metadata).unwrap())
            .audio_id(audio_id),
        )
        .get_result::<Comment>(conn)
        .unwrap()
    })
    .await;

  Response::success(CommentView::new(comment, audio, Some(user)))
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

  let mut audio: Option<Audio> = None;

  if let Some(audio_id) = comment.audio_id {
    audio = Some(
      db_conn
        .run(move |conn| Audio::find_for_user(&user_id, &audio_id).first::<Audio>(conn))
        .await?,
    );
  }

  let audio_id = audio.as_ref().map(|audio| audio.id);

  let updated_comment: Comment = db_conn
    .run(move |conn| {
      diesel::update(&existing_comment)
        .set(
          CommentChangeset::default()
            .body(comment.body.clone())
            .metadata(serde_json::to_value(&comment.metadata).unwrap())
            .audio_id(audio_id),
        )
        .get_result::<Comment>(conn)
        .unwrap()
    })
    .await;

  Response::success(CommentView::new(updated_comment, audio, Some(user)))
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

  let comments: Vec<Comment> = db_conn
    .run(move |conn| {
      Comment::filter_by_recording_id(&recording_id)
        .load::<Comment>(conn)
        .unwrap()
    })
    .await;

  let audio_ids = comments
    .iter()
    .filter_map(|comment| comment.audio_id)
    .collect::<HashSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();

  let audios = db_conn
    .run(move |conn| {
      Audio::filter_processed_by_ids(audio_ids)
        .load::<Audio>(conn)
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
      let audio = audios
        .iter()
        .find(|audio| Some(audio.id) == comment.audio_id)
        .cloned();

      let user = users
        .iter()
        .find(|user| user.id == comment.user_id)
        .cloned();

      CommentView::new(comment, audio, user)
    })
    .collect();

  Response::success(comments)
}
