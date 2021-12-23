use crate::config::Config;
use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Recording, User};
use crate::response::Response;
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
#[post("/recordings", data = "<recording>")]
pub async fn create(
  config: &State<Config>,
  db_conn: DbConn,
  auth: Auth<User>,
  recording: Json<CreateRecordingRequest>,
) -> Response<Recording> {
  if let Err(errors) = recording.validate() {
    return Response::ValidationErrors(errors);
  }

  let extensions = mime_guess::get_mime_extensions_str(&recording.mime_type);

  match extensions {
    None => Response::Status(Status::UnprocessableEntity),
    Some(extensions) => {
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
              user_id.eq(auth.0.id.clone()),
              upload_url.eq(url),
              video_key.eq(key),
              mime_type.eq(recording.mime_type.clone()),
            ))
            .get_result::<Recording>(conn)
            .unwrap()
        })
        .await;

      Response::Success(recording)
    }
  }
}

#[openapi(tag = "Ranklab")]
#[get("/recordings/<id>")]
pub async fn get(
  id: Uuid,
  auth: Auth<User>,
  db_conn: DbConn,
) -> Result<Option<Json<Recording>>, Status> {
  let result = db_conn
    .run(move |conn| {
      use crate::schema::recordings;
      recordings::table.find(id).first::<Recording>(conn)
    })
    .await;

  match result {
    Ok(recording) => {
      if recording.user_id != auth.0.id {
        return Err(Status::Forbidden);
      }

      Ok(Some(Json(recording)))
    }
    Err(diesel::result::Error::NotFound) => Ok(None),
    Err(error) => panic!("Error: {}", error),
  }
}
