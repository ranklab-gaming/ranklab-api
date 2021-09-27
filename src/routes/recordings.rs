use crate::config::Config;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_okapi::{openapi, openapi_get_routes as routes};
use rusoto_core::credential::AwsCredentials;
use rusoto_core::Region;
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::PutObjectRequest;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Recording {
  id: Uuid,
  upload_url: String,
}

#[openapi]
#[post("/")]
fn create_recording(config: &State<Config>) -> Json<Recording> {
  let uuid = Uuid::new_v4();

  let req = PutObjectRequest {
    bucket: config.s3_bucket.to_owned(),
    key: uuid.to_string(),
    ..Default::default()
  };

  let response = req.get_presigned_url(
    &Region::EuWest2,
    &AwsCredentials::new(
      &config.aws_access_key_id,
      &config.aws_secret_key,
      None,
      None,
    ),
    &Default::default(),
  );

  Json(Recording {
    upload_url: response.to_string(),
    id: uuid,
  })
}

pub fn build() -> Vec<Route> {
  routes![create_recording]
}
