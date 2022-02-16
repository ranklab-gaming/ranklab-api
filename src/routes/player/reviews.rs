use crate::guards::Auth;
use crate::guards::DbConn;
use crate::models::{Player, Review};
use crate::response::{QueryResponse, Response};
use crate::views::ReviewView;
use diesel::prelude::*;
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/player/reviews")]
pub async fn list(auth: Auth<Player>, db_conn: DbConn) -> QueryResponse<Vec<ReviewView>> {
  let reviews: Vec<ReviewView> = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews
        .filter(player_id.eq(auth.0.id))
        .load::<Review>(conn)
        .unwrap()
    })
    .await
    .into_iter()
    .map(Into::into)
    .collect();

  Response::success(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/player/reviews/<id>")]
pub async fn get(id: Uuid, auth: Auth<Player>, db_conn: DbConn) -> QueryResponse<ReviewView> {
  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{id as review_id, player_id, reviews};
      reviews
        .filter(player_id.eq(auth.0.id).and(review_id.eq(id)))
        .first::<Review>(conn)
    })
    .await?
    .into();

  Response::success(review)
}
