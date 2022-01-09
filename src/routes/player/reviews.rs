use crate::config::Config;
use crate::db::DbConn;
use crate::emails::{Email, Recipient};
use crate::guards::Auth;
use crate::models::{Coach, Player, Review};
use crate::response::Response;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use validator::{Validate, ValidationError};

fn validate_game_id(game_id: &str) -> Result<(), ValidationError> {
  if crate::games::all().iter().any(|g| g.id() == game_id) {
    Ok(())
  } else {
    Err(ValidationError::new("Game ID is not valid"))
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
#[get("/player/reviews")]
pub async fn list(auth: Auth<Player>, db_conn: DbConn) -> Response<Vec<Review>> {
  let reviews = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews.filter(player_id.eq(auth.0.id)).load(conn).unwrap()
    })
    .await;

  Response::Success(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/player/reviews/<review_id>")]
pub async fn get(review_id: Uuid, auth: Auth<Player>, db_conn: DbConn) -> Response<Review> {
  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews
        .filter(player_id.eq(auth.0.id).and(id.eq(review_id)))
        .first(conn)
    })
    .await?;

  Response::Success(review)
}

#[openapi(tag = "Ranklab")]
#[post("/player/reviews", data = "<review>")]
pub async fn create(
  review: Json<CreateReviewRequest>,
  auth: Auth<Player>,
  db_conn: DbConn,
  config: &State<Config>,
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
          player_id.eq(auth.0.id.clone()),
          notes.eq(review.notes.clone()),
        ))
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  let coaches = db_conn
    .run(move |conn| {
      use crate::schema::coaches::dsl::*;
      coaches.load::<Coach>(conn).unwrap()
    })
    .await;

  let email = Email::new(
    config,
    "notification".to_owned(),
    json!({
        "subject": "New VODs are available",
        "title": "There are new VODs available for review!",
        "body": "Go to your dashboard to start analyzing them.",
        "cta" : "View Available VODs",
        "cta_url" : "https://ranklab.gg/dashboard"
    }),
    coaches
      .iter()
      .map(|coach| {
        Recipient::new(
          coach.email.clone(),
          json!({
            "name": coach.name.clone(),
          }),
        )
      })
      .collect(),
  );

  email.deliver();

  Response::Success(review)
}
