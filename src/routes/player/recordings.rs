use crate::config::Config;
use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Player, Recording};
use crate::response::{MutationError, MutationResponse, QueryResponse, Response};
use diesel::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use rusoto_core::Region;
use rusoto_credential::AwsCredentials;
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::PutObjectRequest;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

lazy_static! {
  static ref MIME_TYPE_REGEX: Regex = Regex::new(r"^video/.*$").unwrap();
}

#[derive(Deserialize, JsonSchema, Validate)]
pub struct CreateRecordingRequest {
  #[validate(range(min = 1usize, max = 2147483648usize))]
  size: usize,
  #[validate(regex = "self::MIME_TYPE_REGEX")]
  mime_type: String,
}

#[openapi(tag = "Ranklab")]
#[post("/player/recordings", data = "<recording>")]
pub async fn create(
  config: &State<Config>,
  db_conn: DbConn,
  auth: Auth<Player>,
  recording: Json<CreateRecordingRequest>,
) -> MutationResponse<Recording> {
  if let Err(errors) = recording.validate() {
    return Response::validation_error(errors);
  }

  let extensions = mime_guess::get_mime_extensions_str(&recording.mime_type)
    .ok_or(MutationError::Status(Status::UnprocessableEntity))?;

  let extension = extensions.first().unwrap();
  let key = format!("{}.{}", Uuid::new_v4().to_string(), extension);

  let req = PutObjectRequest {
    bucket: config.s3_bucket.to_owned(),
    key: key.clone(),
    acl: Some("public-read".to_string()),
    ..Default::default()
  };

  let url = req.get_presigned_url(
    &Region::EuWest2,
    &AwsCredentials::new(
      &config.aws_access_key_id,
      &config.aws_secret_key,
      None,
      None,
    ),
    &Default::default(),
  );

  let recording = db_conn
    .run(move |conn| {
      use crate::schema::recordings::dsl::*;

      diesel::insert_into(recordings)
        .values((
          player_id.eq(auth.0.id.clone()),
          upload_url.eq(url),
          video_key.eq(key),
          mime_type.eq(recording.mime_type.clone()),
        ))
        .get_result::<Recording>(conn)
        .unwrap()
    })
    .await;

  Response::success(recording)
}

#[openapi(tag = "Ranklab")]
#[get("/player/recordings/<id>")]
pub async fn get(id: Uuid, auth: Auth<Player>, db_conn: DbConn) -> QueryResponse<Recording> {
  let recording = db_conn
    .run(move |conn| {
      use crate::schema::recordings::dsl::{id as recording_id, player_id, recordings};
      recordings
        .filter(player_id.eq(auth.0.id).and(recording_id.eq(id)))
        .first(conn)
    })
    .await?;

  Response::success(recording)
}
