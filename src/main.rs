#[macro_use]
extern crate rocket;

use rocket::serde::{json::Json, Serialize};
use rocket::{Build, Rocket};
use rusoto_core::credential::AwsCredentials;
use rusoto_core::Region;
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::PutObjectRequest;
use uuid::Uuid;

#[derive(Serialize)]
struct UrlResponse {
    url: String,
}

#[get("/")]
fn index() -> Json<UrlResponse> {
    let req = PutObjectRequest {
        bucket: "ranklab-development".to_owned(),
        key: Uuid::new_v4().to_string(),
        ..Default::default()
    };

    let presigned_url = req.get_presigned_url(
        &Region::EuWest2,
        &AwsCredentials::new("a", "b", None, None),
        &Default::default(),
    );

    Json(UrlResponse { url: presigned_url })
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![index])
}
