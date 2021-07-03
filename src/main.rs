#[macro_use]
extern crate rocket;

use rocket::fs::TempFile;
use rocket::http::RawStr;
use rocket::serde::{json::Json, Serialize};
use rocket::{Build, Rocket};
use rusoto_core::credential::AwsCredentials;
use rusoto_core::Region;
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::PutObjectRequest;
use uuid::Uuid;

#[derive(Serialize)]
struct Recording {
    id: String,
    upload_url: String,
}

#[post("/recordings")]
fn create_recording() -> Json<Recording> {
    let uuid = Uuid::new_v4();

    let req = PutObjectRequest {
        bucket: "ranklab-development".to_owned(),
        key: uuid.to_string(),
        ..Default::default()
    };

    let upload_url = req.get_presigned_url(
        &Region::EuWest2,
        &AwsCredentials::new("b", "c", None, None),
        &Default::default(),
    );

    Json(Recording {
        upload_url: "http://localhost:8000/upload".to_string(),
        id: uuid.to_string(),
    })
}

#[put("/upload", data = "<input>")]
fn upload(input: TempFile<'_>) -> Json<Recording> {
    Json(Recording {
        upload_url: "".to_string(),
        id: "".to_string(),
    })
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![create_recording, upload])
}
