use crate::config::Config;
use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Game, Review, User};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::serde_json::to_string;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, S3Client, S3};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateReviewRequest {
    recording_id: Uuid,
    #[validate(length(min = 1))]
    title: String,
    game: Game,
}

#[post("/", data = "<review>")]
async fn create_review(
    review: Json<CreateReviewRequest>,
    auth: Auth<User>,
    config: &State<Config>,
    db_conn: DbConn,
) -> Result<Json<Review>, Status> {
    let s3 = S3Client::new(Region::EuWest2);

    if let Err(_) = review.validate() {
        return Err(Status::UnprocessableEntity);
    }

    let get_obj_req = GetObjectRequest {
        bucket: config.s3_bucket.clone(),
        key: review.recording_id.to_string(),
        ..Default::default()
    };

    if let Err(_) = s3.get_object(get_obj_req).await {
        return Err(Status::UnprocessableEntity);
    }

    let video_url_value = format!(
        "https://{}.s3.eu-west-2.amazonaws.com/{}",
        config.s3_bucket,
        review.recording_id.to_string()
    );

    let review = db_conn
        .run(move |conn| {
            use crate::schema::reviews::dsl::*;

            diesel::insert_into(reviews)
                .values((
                    video_url.eq(video_url_value.clone()),
                    title.eq(review.title.clone()),
                    game.eq(to_string(&review.game).unwrap()),
                    user_id.eq(auth.0.id.clone()),
                ))
                .get_result(conn)
                .unwrap()
        })
        .await;

    Ok(Json(review))
}

pub fn build() -> Vec<Route> {
    routes![create_review]
}
