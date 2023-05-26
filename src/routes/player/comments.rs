use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Audio, Comment, Player, Review};
use crate::response::{QueryResponse, Response};
use crate::views::CommentView;
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
  auth: Auth<Jwt<Player>>,
  db_conn: DbConn,
) -> QueryResponse<Vec<CommentView>> {
  let review_id = params.review_id;

  let review: Review = db_conn
    .run(move |conn| Review::find_for_player(&review_id, &auth.into_deep_inner().id).first(conn))
    .await?;

  let audios = db_conn
    .run(move |conn| {
      Audio::filter_by_review_id(&params.review_id)
        .load::<Audio>(conn)
        .unwrap()
    })
    .await;

  let comments: Vec<Comment> = db_conn
    .run(move |conn| Comment::filter_by_review_id(&review.id).load::<Comment>(conn))
    .await?;

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
