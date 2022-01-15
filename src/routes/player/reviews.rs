use crate::config::Config;
use crate::db::DbConn;
use crate::emails::{Email, Recipient};
use crate::guards::Auth;
use crate::models::{Coach, Player, Review};
use crate::response::{MutationResponse, QueryResponse, Response};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateReviewRequest {
  recording_id: Uuid,
  #[validate(length(min = 1))]
  title: String,
  notes: String,
  #[validate(custom = "crate::games::validate_id")]
  game_id: String,
}

#[openapi(tag = "Ranklab")]
#[get("/player/reviews")]
pub async fn list(auth: Auth<Player>, db_conn: DbConn) -> QueryResponse<Vec<Review>> {
  let reviews = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews.filter(player_id.eq(auth.0.id)).load(conn).unwrap()
    })
    .await;

  Response::success(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/player/reviews/<id>")]
pub async fn get(id: Uuid, auth: Auth<Player>, db_conn: DbConn) -> QueryResponse<Review> {
  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{id as review_id, player_id, reviews};
      reviews
        .filter(player_id.eq(auth.0.id).and(review_id.eq(id)))
        .first(conn)
    })
    .await?;

  Response::success(review)
}

#[openapi(tag = "Ranklab")]
#[post("/player/reviews", data = "<review>")]
pub async fn create(
  review: Json<CreateReviewRequest>,
  auth: Auth<Player>,
  db_conn: DbConn,
  config: &State<Config>,
) -> MutationResponse<Review> {
  if let Err(errors) = review.validate() {
    return Response::validation_error(errors);
  }

  let game = auth
    .0
    .games
    .clone()
    .into_iter()
    .find(|g| g.game_id == review.game_id);

  if game.is_none() {
    return Response::mutation_error(Status::BadRequest);
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
          skill_level.eq(game.unwrap().skill_level as i16),
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

  Response::success(review)
}
