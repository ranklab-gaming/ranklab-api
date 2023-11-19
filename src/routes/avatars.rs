use crate::config::Config;
use crate::guards::{Auth, DbConn, Jwt, S3};
use crate::models::{Avatar, AvatarChangeset};
use crate::response::{MutationResponse, QueryResponse, Response, StatusResponse};
use crate::schema::avatars;
use crate::views::AvatarView;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::State;
use rocket_okapi::openapi;
use rusoto_core::Region;
use rusoto_credential::AwsCredentials;
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::{Delete, DeleteObjectsRequest, ObjectIdentifier, PutObjectRequest, S3 as RusotoS3};
use std::collections::HashMap;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[post("/avatars")]
pub async fn create(
  config: &State<Config>,
  db_conn: DbConn,
  auth: Auth<Jwt>,
) -> MutationResponse<AvatarView> {
  let user = auth.into_user();
  let key = format!("avatars/originals/{}", Uuid::new_v4());

  let avatar = db_conn
    .run(move |conn| {
      diesel::insert_into(avatars::table)
        .values(AvatarChangeset::default().image_key(key).user_id(user.id))
        .get_result::<Avatar>(conn)
        .unwrap()
    })
    .await;

  let mut metadata = HashMap::new();

  if let Some(instance_id) = config.instance_id.as_ref() {
    metadata.insert("instance-id".to_string(), instance_id.to_string());
  }

  let req = PutObjectRequest {
    bucket: config.uploads_bucket.to_owned(),
    key: avatar.image_key.to_owned(),
    acl: Some("public-read".to_string()),
    metadata: Some(metadata),
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

  Response::success(AvatarView::new(
    avatar,
    Some(url),
    config.instance_id.clone(),
  ))
}

#[openapi(tag = "Ranklab")]
#[delete("/avatars")]
pub async fn delete(
  db_conn: DbConn,
  auth: Auth<Jwt>,
  config: &State<Config>,
  s3: S3,
) -> MutationResponse<StatusResponse> {
  let user = auth.into_user();
  let s3 = s3.into_inner();

  let avatar = db_conn
    .run(move |conn| Avatar::find_for_user(&user.id).first::<Avatar>(conn))
    .await?;

  let req = DeleteObjectsRequest {
    bucket: config.uploads_bucket.to_owned(),
    delete: Delete {
      objects: vec![
        ObjectIdentifier {
          key: avatar.image_key.clone(),
          ..Default::default()
        },
        ObjectIdentifier {
          key: avatar.processed_image_key.clone().unwrap(),
          ..Default::default()
        },
      ],
      ..Default::default()
    },
    ..Default::default()
  };

  s3.delete_objects(req).await.unwrap();

  db_conn
    .run(move |conn| diesel::delete(&avatar).execute(conn).unwrap())
    .await;

  Response::status(Status::NoContent)
}

#[openapi(tag = "Ranklab")]
#[get("/avatars/<id>")]
pub async fn get(auth: Auth<Jwt>, id: Uuid, db_conn: DbConn) -> QueryResponse<AvatarView> {
  let user_id = auth.into_user().id;

  let avatar = db_conn
    .run(move |conn| Avatar::find_by_id_for_user(&id, &user_id).first::<Avatar>(conn))
    .await?;

  Response::success(AvatarView::new(avatar, None, None))
}
