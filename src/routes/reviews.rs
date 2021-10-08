use crate::config::Config;
use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Review, User};
use crate::response::Response;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, S3Client, S3};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateReviewRequest {
  recording_id: Uuid,
  #[validate(length(min = 1))]
  title: String,
  game_id: Uuid,
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
          game_id.eq(review.game_id.clone()),
          user_id.eq(auth.0.id.clone()),
        ))
        .get_result(conn)
        .unwrap()
    })
    .await;

  Response::Success(review)
}
