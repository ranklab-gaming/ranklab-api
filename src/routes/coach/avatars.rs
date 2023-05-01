use crate::config::Config;
use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Avatar, AvatarChangeset, Coach};
use crate::response::{MutationResponse, Response, StatusResponse};
use crate::schema::avatars;
use crate::views::AvatarView;
use diesel::prelude::*;
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

#[derive(Deserialize, JsonSchema)]
pub struct CreateAvatarRequest {}

#[openapi(tag = "Ranklab")]
#[post("/coach/avatars", data = "<_avatar>")]
pub async fn create(
  config: &State<Config>,
  db_conn: DbConn,
  auth: Auth<Jwt<Coach>>,
  _avatar: Json<CreateAvatarRequest>,
) -> MutationResponse<AvatarView> {
  let coach = auth.into_deep_inner();
  let key = format!("avatars/originals/{}", Uuid::new_v4());

  let avatar: Avatar = db_conn
    .run(move |conn| {
      diesel::insert_into(avatars::table)
        .values(AvatarChangeset::default().image_key(key).coach_id(coach.id))
        .get_result::<Avatar>(conn)
        .unwrap()
    })
    .await;

  let req = PutObjectRequest {
    bucket: config.s3_bucket.to_owned(),
    key: avatar.image_key.to_owned(),
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

  Response::success(AvatarView::new(avatar, Some(url)))
}

#[openapi(tag = "Ranklab")]
#[delete("/coach/avatars")]
pub async fn delete(db_conn: DbConn, auth: Auth<Jwt<Coach>>) -> MutationResponse<StatusResponse> {
  let coach = auth.into_deep_inner();

  if let Some(avatar_id) = coach.avatar_id {
    db_conn
      .run(move |conn| {
        diesel::delete(Avatar::find_by_id(&avatar_id))
          .execute(conn)
          .unwrap()
      })
      .await;
  }

  Response::status(Status::NoContent)
}
