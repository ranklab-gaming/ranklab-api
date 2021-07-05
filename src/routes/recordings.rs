use crate::models::Recording;
use rocket::serde::json::Json;
use rocket::Route;
use rusoto_core::credential::AwsCredentials;
use rusoto_core::Region;
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::PutObjectRequest;
use uuid::Uuid;

#[post("/")]
fn create_recording() -> Json<Recording> {
  let uuid = Uuid::new_v4();

  let req = PutObjectRequest {
    bucket: "ranklab-development".to_owned(),
    key: uuid.to_string(),
    ..Default::default()
  };

  let _ = req.get_presigned_url(
    &Region::EuWest2,
    &AwsCredentials::new("a", "b", None, None),
    &Default::default(),
  );

  Json(Recording {
    upload_url: "http://localhost:8000/upload".to_string(),
    id: uuid.to_string(),
  })
}

pub fn build() -> Vec<Route> {
  routes![create_recording]
}
