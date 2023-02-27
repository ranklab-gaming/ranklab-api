use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Comment, Player, Review};
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
  let review: Review = db_conn
    .run(move |conn| {
      Review::find_for_player(&params.review_id, &auth.into_deep_inner().id).first(conn)
    })
    .await?;

  let comments: Vec<CommentView> = db_conn
    .run(move |conn| Comment::filter_by_review_id(&review.id).load::<Comment>(conn))
    .await?
    .into_iter()
    .map(Into::into)
    .collect();

  Response::success(comments)
}
