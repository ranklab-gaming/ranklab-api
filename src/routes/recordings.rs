use crate::config::Config;
use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::User;
use crate::response::Response;
use lazy_static::lazy_static;
use regex::Regex;
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
use validator::Validate;

lazy_static! {
  static ref MIME_TYPE_REGEX: Regex = Regex::new(r"^video/.*$").unwrap();
}

#[derive(Deserialize, JsonSchema, Validate)]
pub struct CreateRecordingRequest {
  extension: String,
  size: usize,
  #[validate(regex = "self::MIME_TYPE_REGEX")]
  mime_type: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CreateRecordingResponse {
  id: Uuid,
  upload_url: String,
}

#[openapi(tag = "Ranklab")]
#[post("/recordings", data = "<recording>")]
pub async fn create(
  config: &State<Config>,
  db_conn: DbConn,
  auth: Auth<User>,
  recording: Json<CreateRecordingRequest>,
) -> Response<CreateRecordingResponse> {
  if let Err(errors) = recording.validate() {
    return Response::ValidationErrors(errors);
  }

  let recording = db_conn
    .run(move |conn| {
      use crate::schema::recordings::dsl::*;

      diesel::insert_into(recordings)
        .values((user_id.eq(auth.0.id.clone()), extension.eq("123")))
        .get_result(conn)
        .unwrap()
    })
    .await;

  let req = PutObjectRequest {
    bucket: config.s3_bucket.to_owned(),
    key: format!("{}", recording.id.to_string()),
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

  Json(CreateRecordingResponse {
    upload_url: response.to_string(),
    id: uuid,
  })
}
