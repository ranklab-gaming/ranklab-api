use crate::config::Config;
use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Game, Review, User};
use crate::response::Response;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::serde_json::to_string;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_okapi::{openapi, openapi_get_routes as routes};
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, S3Client, S3};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, JsonSchema)]
struct CreateReviewRequest {
  recording_id: Uuid,
  #[validate(length(min = 1))]
  title: String,
  game: Game,
}

#[openapi]
#[get("/")]
async fn list_reviews(auth: Auth<User>, db_conn: DbConn) -> Json<Vec<Review>> {
  let reviews = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews.filter(user_id.eq(auth.0.id)).load(conn).unwrap()
    })
    .await;

  Json(reviews)
}

#[openapi]
#[get("/<id>")]
async fn get_review(
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

#[openapi]
#[post("/", data = "<review>")]
async fn create_review(
  review: Json<CreateReviewRequest>,
  auth: Auth<User>,
  config: &State<Config>,
  db_conn: DbConn,
) -> Response<Review> {
  let s3 = S3Client::new(Region::EuWest2);

  if let Err(errors) = review.validate() {
    return Response::ValidationErrors(errors);
  }

  let get_obj_req = GetObjectRequest {
    bucket: config.s3_bucket.clone(),
    key: review.recording_id.to_string(),
    ..Default::default()
  };

  if let Err(_) = s3.get_object(get_obj_req).await {
    return Response::Status(Status::UnprocessableEntity);
  }

  let video_url_value = format!(
    "https://{}.s3.eu-west-2.amazonaws.com/{}",
    config.s3_bucket,
    review.recording_id.to_string()
  );

  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;

      diesel::insert_into(reviews)
        .values((
          video_url.eq(video_url_value.clone()),
          title.eq(review.title.clone()),
          game.eq(to_string(&review.game).unwrap()),
          user_id.eq(auth.0.id.clone()),
        ))
        .get_result(conn)
        .unwrap()
    })
    .await;

  Response::Success(review)
}

pub fn build() -> Vec<Route> {
  routes![create_review, list_reviews, get_review]
}
