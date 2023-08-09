use crate::games::GameId;
use crate::guards::DbConn;
use crate::models::{Comment, Recording, RecordingWithCommentCount, User};
use crate::pagination::{Paginate, PaginatedResult};
use crate::response::{QueryResponse, Response};
use crate::views::{CommentView, RecordingView, RecordingWithCommentCountView};
use diesel::prelude::*;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(FromForm, JsonSchema)]
pub struct ListParams {
  page: Option<i64>,
  game_id: Option<GameId>,
}

#[openapi(tag = "Ranklab")]
#[get("/explore?<params..>")]
pub async fn list(
  db_conn: DbConn,
  params: ListParams,
) -> QueryResponse<PaginatedResult<RecordingWithCommentCountView>> {
  let page = params.page.unwrap_or(1);

  let recordings = db_conn
    .run(move |conn| match params.game_id {
      Some(game_id) => Recording::filter_by_game_id(&game_id.to_string())
        .paginate(page)
        .load_and_count_pages::<RecordingWithCommentCount>(conn)
        .unwrap(),
      None => Recording::all()
        .paginate(page)
        .load_and_count_pages::<RecordingWithCommentCount>(conn)
        .unwrap(),
    })
    .await;

  let user_ids = recordings
    .records
    .clone()
    .into_iter()
    .map(|recording| recording.recording.user_id)
    .collect::<HashSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();

  let users = db_conn
    .run(move |conn| {
      User::filter_by_ids(user_ids)
        .load::<crate::models::User>(conn)
        .unwrap()
    })
    .await;

  let recording_views = recordings
    .records
    .clone()
    .into_iter()
    .map(|recording| {
      let user = users
        .clone()
        .into_iter()
        .find(|user| user.id == recording.recording.user_id)
        .unwrap();

      RecordingWithCommentCountView::new(recording, None, None, Some(user))
    })
    .collect::<Vec<RecordingWithCommentCountView>>();

  Response::success(recordings.records(recording_views))
}

#[openapi(tag = "Ranklab")]
#[get("/explore/<id>")]
pub async fn get(id: Uuid, db_conn: DbConn) -> QueryResponse<RecordingView> {
  let recording: Recording = db_conn
    .run(move |conn| Recording::find_by_id(&id).first::<Recording>(conn))
    .await?;

  let user_id = recording.user_id;

  let user = db_conn
    .run(move |conn| User::find_by_id(&user_id).first::<User>(conn))
    .await?;

  Response::success(RecordingView::new(recording, None, None, Some(user)))
}

#[openapi(tag = "Ranklab")]
#[get("/explore/<id>/comments")]
pub async fn get_comments(id: Uuid, db_conn: DbConn) -> QueryResponse<Vec<CommentView>> {
  let comments = db_conn
    .run(move |conn| Comment::filter_by_recording_id(&id).load::<Comment>(conn))
    .await?;

  let user_ids = comments
    .iter()
    .map(|comment| comment.user_id)
    .collect::<HashSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();

  let users = db_conn
    .run(move |conn| User::filter_by_ids(user_ids).load::<User>(conn).unwrap())
    .await;

  let comment_views = comments
    .clone()
    .into_iter()
    .map(|comment| {
      let user = users
        .clone()
        .into_iter()
        .find(|user| user.id == comment.user_id);

      CommentView::new(comment, None, user)
    })
    .collect::<Vec<CommentView>>();

  Response::success(comment_views)
}
