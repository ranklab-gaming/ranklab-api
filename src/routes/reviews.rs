use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Review, User};
use crate::response::Response;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;
use validator::{Validate, ValidationError};

fn validate_game_id(game_id: &str) -> Result<(), ValidationError> {
  if crate::games::all().iter().any(|g| g.id() == game_id) {
    Err(ValidationError::new("Game ID is not valid"))
  } else {
    Ok(())
  }
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateReviewRequest {
  recording_id: Uuid,
  #[validate(length(min = 1))]
  title: String,
  notes: String,
  #[validate(custom = "validate_game_id")]
  game_id: String,
}

#[openapi(tag = "Ranklab")]
#[get("/reviews")]
pub async fn list(auth: Auth<User>, db_conn: DbConn) -> Json<Vec<Review>> {
  let reviews = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews.filter(user_id.eq(auth.0.id)).load(conn).unwrap()
    })
    .await;

  Json(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/reviews/<id>")]
pub async fn get(
  id: Uuid,
  auth: Auth<User>,
  db_conn: DbConn,
) -> Result<Option<Json<Review>>, Status> {
  let result = db_conn
    .run(move |conn| {
      use crate::schema::reviews;
      reviews::table.find(id).first::<Review>(conn)
    })
    .await;

  match result {
    Ok(review) => {
      if review.user_id != auth.0.id {
        return Err(Status::Forbidden);
      }

      Ok(Some(Json(review)))
    }
    Err(diesel::result::Error::NotFound) => Ok(None),
    Err(error) => panic!("Error: {}", error),
  }
}

#[openapi(tag = "Ranklab")]
#[post("/reviews", data = "<review>")]
pub async fn create(
  review: Json<CreateReviewRequest>,
  auth: Auth<User>,
  db_conn: DbConn,
) -> Response<Review> {
  if let Err(errors) = review.validate() {
    return Response::ValidationErrors(errors);
  }

  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;

      diesel::insert_into(reviews)
        .values((
          recording_id.eq(review.recording_id.clone()),
          title.eq(review.title.clone()),
          game_id.eq(review.game_id.clone()),
          user_id.eq(auth.0.id.clone()),
          notes.eq(review.notes.clone()),
        ))
        .get_result(conn)
        .unwrap()
    })
    .await;

  Response::Success(review)
}
