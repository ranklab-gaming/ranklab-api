use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Audio, Coach, Comment, CommentChangeset, Review};
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
  review_id: Uuid,
  metadata: serde_json::Value,
  audio_id: Option<Uuid>,
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct UpdateCommentRequest {
  body: String,
  metadata: serde_json::Value,
  audio_id: Option<Uuid>,
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

  let mut audio: Option<Audio> = None;
  let review_id = review.id;

  if let Some(audio_id) = comment.audio_id {
    audio = Some(
      db_conn
        .run(move |conn| Audio::find_for_review_id(&audio_id, &review_id).first::<Audio>(conn))
        .await?,
    );
  }

  let comment: CommentView = db_conn
    .run(move |conn| {
      diesel::insert_into(comments::table)
        .values(
          CommentChangeset::default()
            .body(ammonia::clean(&comment.body))
            .review_id(review.id)
            .coach_id(coach_id)
            .metadata(comment.metadata.clone())
            .audio_id(audio.map(|a| a.id)),
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

  let mut audio: Option<Audio> = None;
  let review_id = existing_comment.review_id;

  if let Some(audio_id) = comment.audio_id {
    audio = Some(
      db_conn
        .run(move |conn| Audio::find_for_review_id(&audio_id, &review_id).first::<Audio>(conn))
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
            .metadata(comment.metadata.clone())
            .audio_id(audio_id),
        )
        .get_result::<Comment>(conn)
        .unwrap()
    })
    .await;

  Response::success(CommentView::new(updated_comment, audio))
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
  let review_id = params.review_id;

  let comments: Vec<Comment> = db_conn
    .run(move |conn| {
      Comment::filter_by_review_for_coach(&review_id, &auth.into_deep_inner().id)
        .load::<Comment>(conn)
        .unwrap()
    })
    .await;

  let audios = db_conn
    .run(move |conn| {
      Audio::filter_by_review_id(&params.review_id)
        .load::<Audio>(conn)
        .unwrap()
    })
    .await;

  let comments = comments
    .into_iter()
    .map(|comment| {
      let audio = audios
        .iter()
        .find(|audio| Some(audio.id) == comment.audio_id)
        .cloned();

      CommentView::new(comment, audio)
    })
    .collect();

  Response::success(comments)
}
