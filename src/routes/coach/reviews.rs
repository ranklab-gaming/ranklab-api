use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Coach, Review};
use crate::response::Response;
use diesel::prelude::*;
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews")]
pub async fn list(auth: Auth<Coach>, db_conn: DbConn) -> Response<Vec<Review>> {
  let reviews = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews.filter(coach_id.eq(auth.0.id)).load(conn).unwrap()
    })
    .await;

  Response::Success(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews/<review_id>")]
pub async fn get(review_id: Uuid, auth: Auth<Coach>, db_conn: DbConn) -> Response<Review> {
  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews
        .filter(coach_id.eq(auth.0.id).and(id.eq(review_id)))
        .first(conn)
    })
    .await?;

  Response::Success(review)
}
