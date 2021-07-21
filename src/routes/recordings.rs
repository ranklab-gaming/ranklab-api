use crate::models::Recording;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rusoto_core::credential::AwsCredentials;
use rusoto_core::Region;
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::PutObjectRequest;
use uuid::Uuid;
use crate::config::Config;

#[post("/")]
fn create_recording(
  config: &State<Config>
) -> Json<Recording> {
  let uuid = Uuid::new_v4();

  let req = PutObjectRequest {
    bucket: config.s3_bucket.to_owned(),
    key: uuid.to_string(),
    ..Default::default()
  };

  let response = req.get_presigned_url(
    &Region::EuWest2,
    &AwsCredentials::new(config.aws_access_key_id, config.aws_secret_key, None, None),
    &Default::default(),
  );

  Json(Recording {
    upload_url: response.to_string(),
    id: uuid.to_string(),
  })
}

pub fn build() -> Vec<Route> {
  routes![create_recording]
}