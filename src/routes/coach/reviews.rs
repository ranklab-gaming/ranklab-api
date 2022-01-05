use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Coach, Review};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews")]
pub async fn list(auth: Auth<Coach>, db_conn: DbConn) -> Json<Vec<Review>> {
  let reviews = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews.filter(coach_id.eq(auth.0.id)).load(conn).unwrap()
    })
    .await;

  Json(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews/<id>")]
pub async fn get(
  id: Uuid,
  auth: Auth<Coach>,
  db_conn: DbConn,
) -> Result<Option<Json<Review>>, Status> {
  let result = db_conn
    .run(move |conn| {
      use crate::schema::reviews;
      reviews::table.find(id).first::<Review>(conn)
    })
    .await;

  match result {
    Ok(review) => match review.coach_id {
      Some(coach_id) => {
        if coach_id == auth.0.id {
          Ok(Some(Json(review)))
        } else {
          Err(Status::Forbidden)
        }
      }
      _ => Err(Status::Forbidden),
    },
    Err(diesel::result::Error::NotFound) => Ok(None),
    Err(error) => panic!("Error: {}", error),
  }
}
