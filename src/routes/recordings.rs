use crate::config::Config;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use rusoto_core::Region;
use rusoto_credential::AwsCredentials;
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::PutObjectRequest;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Recording {
  id: Uuid,
  upload_url: String,
}

#[openapi(tag = "Ranklab")]
#[post("/recordings")]
pub fn create(config: &State<Config>) -> Json<Recording> {
  let uuid = Uuid::new_v4();

  let req = PutObjectRequest {
    bucket: config.s3_bucket.to_owned(),
    key: uuid.to_string(),
    acl: Some("public-read".to_string()),
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
