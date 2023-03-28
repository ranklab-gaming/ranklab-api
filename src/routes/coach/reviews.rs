use crate::config::Config;
use crate::data_types::ReviewState;
use crate::emails::{Email, Recipient};
use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Coach, Player, Recording, Review, ReviewChangeset};
use crate::pagination::{Paginate, PaginatedResult};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::views::{ReviewView, ReviewViewOptions};
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

#[derive(FromForm, JsonSchema)]
pub struct ListReviewsQuery {
  page: Option<i64>,
  archived: Option<bool>,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews?<params..>")]
pub async fn list(
  auth: Auth<Jwt<Coach>>,
  db_conn: DbConn,
  params: ListReviewsQuery,
) -> QueryResponse<PaginatedResult<ReviewView>> {
  let coach = auth.into_deep_inner();

  let paginated_reviews: PaginatedResult<Review> = db_conn
    .run(move |conn| {
      Review::filter_for_coach(&coach, params.archived.unwrap_or(false))
        .paginate(params.page.unwrap_or(1))
        .load_and_count_pages::<Review>(conn)
        .unwrap()
    })
    .await;

  let records = paginated_reviews.records.clone();

  let recordings = db_conn
    .run(move |conn| {
      Recording::filter_by_ids(
        records
          .into_iter()
          .map(|review| review.recording_id)
          .collect(),
      )
      .load::<Recording>(conn)
    })
    .await?;

  let records = paginated_reviews.records.clone();

  let players = db_conn
    .run(move |conn| {
      Player::filter_by_ids(records.into_iter().map(|review| review.player_id).collect())
        .load::<Player>(conn)
    })
    .await?;

  let review_views: Vec<ReviewView> = paginated_reviews
    .records
    .clone()
    .into_iter()
    .map(|review| {
      let recording_id = review.recording_id;
      let player_id = review.player_id;

      ReviewView::new(
        review,
        ReviewViewOptions {
          payment_intent: None,
          tax_calculation: None,
          coach: None,
          recording: recordings
            .iter()
            .find(|recording| recording.id == recording_id)
            .cloned(),
          player: players
            .iter()
            .find(|player| player.id == player_id)
            .cloned(),
        },
      )
    })
    .collect();

  Response::success(paginated_reviews.records(review_views))
}

#[derive(Deserialize, Validate, JsonSchema)]
#[schemars(rename = "CoachUpdateReviewRequest")]
pub struct UpdateReviewRequest {
  published: Option<bool>,
  started: Option<bool>,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews/<id>")]
pub async fn get(id: Uuid, auth: Auth<Jwt<Coach>>, db_conn: DbConn) -> QueryResponse<ReviewView> {
  let coach = auth.into_deep_inner();

  let review = db_conn
    .run(move |conn| Review::find_for_coach(&id, &coach.id).first::<Review>(conn))
    .await?;

  let recording_id = review.recording_id;
  let player_id = review.player_id;

  let player = db_conn
    .run(move |conn| Player::find_by_id(&player_id).first::<Player>(conn))
    .await?;

  let recording = db_conn
    .run(move |conn| Recording::find_by_id(&recording_id).first::<Recording>(conn))
    .await?;

  Response::success(ReviewView::new(
    review,
    ReviewViewOptions {
      payment_intent: None,
      tax_calculation: None,
      coach: None,
      recording: Some(recording),
      player: Some(player),
    },
  ))
}

#[openapi(tag = "Ranklab")]
#[put("/coach/reviews/<id>", data = "<review>")]
pub async fn update(
  id: Uuid,
  review: Json<UpdateReviewRequest>,
  auth: Auth<Jwt<Coach>>,
  db_conn: DbConn,
  config: &State<Config>,
) -> MutationResponse<ReviewView> {
  if let Err(errors) = review.validate() {
    return Response::validation_error(errors);
  }

  let coach = auth.into_deep_inner();
  let coach_id = coach.id;

  let existing_review = db_conn
    .run(move |conn| Review::find_for_coach(&id, &coach_id).first::<Review>(conn))
    .await?;

  let player_id = existing_review.player_id;

  let player = db_conn
    .run(move |conn| Player::find_by_id(&player_id).first::<Player>(conn))
    .await?;

  let recording_id = existing_review.recording_id;

  let recording = db_conn
    .run(move |conn| Recording::find_by_id(&recording_id).first::<Recording>(conn))
    .await?;

  if let Some(true) = review.published {
    if existing_review.state == ReviewState::Draft {
      let updated_review = db_conn
        .run(move |conn| {
          diesel::update(&existing_review)
            .set(ReviewChangeset::default().state(ReviewState::Published))
            .get_result::<Review>(conn)
            .unwrap()
        })
        .await;

      let email = Email::new(
        config,
        "notification".to_owned(),
        json!({
          "subject": "Your review has been completed",
          "title": format!("{} has finished reviewing your recording.", coach.name),
          "body": "You can now have a look at the comments and suggestions your coach has made.",
          "cta" : "View Review",
          "cta_url" : format!("{}/player/reviews/{}", config.web_host, updated_review.id),
          "unsubscribe_url": format!("{}/player/account?tab=notifications", config.web_host),
        }),
        vec![Recipient::new(
          player.email.clone(),
          json!({
            "name": player.name,
          }),
        )],
      );

      email.deliver().await.unwrap();

      return Response::success(ReviewView::new(
        updated_review,
        ReviewViewOptions {
          payment_intent: None,
          tax_calculation: None,
          coach: None,
          recording: Some(recording),
          player: Some(player),
        },
      ));
    }
  }

  if let Some(true) = review.started {
    if existing_review.state == ReviewState::AwaitingReview {
      let updated_review = db_conn
        .run(move |conn| {
          diesel::update(&existing_review)
            .set(ReviewChangeset::default().state(ReviewState::Draft))
            .get_result::<Review>(conn)
            .unwrap()
        })
        .await;

      let player_id = updated_review.player_id;

      let player = db_conn
        .run(move |conn| Player::find_by_id(&player_id).first::<Player>(conn))
        .await?;

      let email = Email::new(
        config,
        "notification".to_owned(),
        json!({
          "subject": format!("{} has started reviewing your recording", coach.name),
          "title": format!("{} has started reviewing your recording", coach.name),
          "body": "You will receive an email when your coach is finished with the review.",
          "cta" : "View Review",
          "cta_url" : format!("{}/player/reviews/{}", config.web_host, updated_review.id),
          "unsubscribe_url": format!("{}/player/account?tab=notifications", config.web_host),
        }),
        vec![Recipient::new(
          player.email.clone(),
          json!({
            "name": player.name,
          }),
        )],
      );

      email.deliver().await.unwrap();

      return Response::success(ReviewView::new(
        updated_review,
        ReviewViewOptions {
          payment_intent: None,
          tax_calculation: None,
          coach: None,
          recording: Some(recording),
          player: Some(player),
        },
      ));
    }
  }

  Response::success(ReviewView::new(
    existing_review,
    ReviewViewOptions {
      payment_intent: None,
      tax_calculation: None,
      coach: None,
      recording: Some(recording),
      player: Some(player),
    },
  ))
}
