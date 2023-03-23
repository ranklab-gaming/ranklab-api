use crate::config::Config;
use crate::games;
use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Player, Recording, RecordingChangeset};
use crate::pagination::{Paginate, PaginatedResult};
use crate::response::{MutationError, MutationResponse, QueryResponse, Response};
use crate::schema::recordings;
use crate::views::RecordingView;
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
  #[validate(length(min = 1))]
  title: String,
  skill_level: i16,
  #[validate(length(min = 1), custom = "games::validate_id")]
  game_id: String,
}

#[derive(FromForm, JsonSchema)]
pub struct ListRecordingsQuery {
  page: Option<i64>,
}

#[openapi(tag = "Ranklab")]
#[get("/player/recordings?<params..>")]
pub async fn list(
  auth: Auth<Jwt<Player>>,
  db_conn: DbConn,
  params: ListRecordingsQuery,
) -> QueryResponse<PaginatedResult<RecordingView>> {
  let recordings: PaginatedResult<Recording> = db_conn
    .run(move |conn| {
      Recording::filter_for_player(&auth.into_deep_inner().id)
        .paginate(params.page.unwrap_or(1))
        .load_and_count_pages::<Recording>(conn)
        .unwrap()
    })
    .await;

  let recording_views: Vec<RecordingView> = recordings
    .records
    .clone()
    .into_iter()
    .map(|recording| RecordingView::new(recording, None))
    .collect();

  Response::success(recordings.records(recording_views))
}

#[openapi(tag = "Ranklab")]
#[post("/player/recordings", data = "<recording>")]
pub async fn create(
  config: &State<Config>,
  db_conn: DbConn,
  auth: Auth<Jwt<Player>>,
  recording: Json<CreateRecordingRequest>,
) -> MutationResponse<RecordingView> {
  if let Err(errors) = recording.validate() {
    return Response::validation_error(errors);
  }

  let game = games::find(&recording.game_id).unwrap();

  if !game
    .skill_levels
    .iter()
    .any(|skill_level| skill_level.value == recording.skill_level as u8)
  {
    return Response::mutation_error(Status::UnprocessableEntity);
  }

  let extensions = mime_guess::get_mime_extensions_str(&recording.mime_type)
    .ok_or(MutationError::Status(Status::UnprocessableEntity))?;

  let extension = extensions.first().unwrap();
  let key = format!("{}.{}", Uuid::new_v4(), extension);

  let recording: Recording = db_conn
    .run(move |conn| {
      diesel::insert_into(recordings::table)
        .values(
          RecordingChangeset::default()
            .player_id(auth.into_deep_inner().id)
            .game_id(recording.game_id.clone())
            .title(recording.title.clone())
            .skill_level(recording.skill_level)
            .video_key(key)
            .mime_type(recording.mime_type.clone()),
        )
        .get_result::<Recording>(conn)
        .unwrap()
    })
    .await;

  let url = create_upload_url(config, &recording.video_key);

  Response::success(RecordingView::new(recording, Some(url)))
}

#[openapi(tag = "Ranklab")]
#[get("/player/recordings/<id>")]
pub async fn get(
  id: Uuid,
  auth: Auth<Jwt<Player>>,
  db_conn: DbConn,
  config: &State<Config>,
) -> QueryResponse<RecordingView> {
  let recording: Recording = db_conn
    .run(move |conn| {
      Recording::find_for_player(&id, &auth.into_deep_inner().id).first::<Recording>(conn)
    })
    .await?;

  let url = create_upload_url(config, &recording.video_key);

  Response::success(RecordingView::new(recording, Some(url)))
}

fn create_upload_url(config: &Config, key: &str) -> String {
  let req = PutObjectRequest {
    bucket: config.s3_bucket.to_owned(),
    key: key.to_owned(),
    acl: Some("public-read".to_string()),
    ..Default::default()
  };

  req.get_presigned_url(
    &Region::EuWest2,
    &AwsCredentials::new(
      &config.aws_access_key_id,
      &config.aws_secret_key,
      None,
      None,
    ),
    &Default::default(),
  )
}
