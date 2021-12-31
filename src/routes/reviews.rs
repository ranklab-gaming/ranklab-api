use crate::aws;
use crate::config::Config;
use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Coach, Player, Review};
use crate::response::Response;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::tokio;
use rocket::State;
use rocket_okapi::openapi;
use rusoto_core::HttpClient;
use rusoto_core::Region;
use rusoto_sesv2::{
  BulkEmailContent, BulkEmailEntry, Destination, ReplacementEmailContent, ReplacementTemplate,
  SendBulkEmailRequest, SesV2, SesV2Client, Template,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use validator::{Validate, ValidationError};

fn validate_game_id(game_id: &str) -> Result<(), ValidationError> {
  if crate::games::all().iter().any(|g| g.id() == game_id) {
    Ok(())
  } else {
    Err(ValidationError::new("Game ID is not valid"))
  }
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateReviewRequest {
  recording_id: Uuid,
  #[validate(length(min = 1))]
  title: String,
  notes: String,
  #[validate(custom = "validate_game_id")]
  game_id: String,
}

#[openapi(tag = "Ranklab")]
#[get("/reviews")]
pub async fn list(auth: Auth<Coach>, db_conn: DbConn) -> Json<Vec<Review>> {
  let reviews = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews.filter(player_id.eq(auth.0.id)).load(conn).unwrap()
    })
    .await;

  Json(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/reviews/<id>")]
pub async fn get(
  id: Uuid,
  auth: Auth<Coach>,
  db_conn: DbConn,
) -> Result<Option<Json<Review>>, Status> {
  let result = db_conn
    .run(move |conn| {
      use crate::schema::reviews;
      reviews::table.find(id).first::<Review>(conn)
    })
    .await;

  match result {
    Ok(review) => {
      if review.player_id != auth.0.id {
        return Err(Status::Forbidden);
      }

      Ok(Some(Json(review)))
    }
    Err(diesel::result::Error::NotFound) => Ok(None),
    Err(error) => panic!("Error: {}", error),
  }
}

#[openapi(tag = "Ranklab")]
#[post("/reviews", data = "<review>")]
pub async fn create(
  review: Json<CreateReviewRequest>,
  auth: Auth<Player>,
  db_conn: DbConn,
  config: &State<Config>,
) -> Response<Review> {
  if let Err(errors) = review.validate() {
    return Response::ValidationErrors(errors);
  }

  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;

      diesel::insert_into(reviews)
        .values((
          recording_id.eq(review.recording_id.clone()),
          title.eq(review.title.clone()),
          game_id.eq(review.game_id.clone()),
          player_id.eq(auth.0.id.clone()),
          notes.eq(review.notes.clone()),
        ))
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  let aws_access_key_id = config.aws_access_key_id.clone();
  let aws_secret_key = config.aws_secret_key.clone();

  tokio::spawn(async move {
    let coaches = db_conn
      .run(move |conn| {
        use crate::schema::coaches::dsl::*;
        coaches.load::<Coach>(conn).unwrap()
      })
      .await;

    let client = SesV2Client::new_with(
      HttpClient::new().unwrap(),
      aws::CredentialsProvider::new(aws_access_key_id, aws_secret_key),
      Region::EuWest2,
    );

    let email_request = SendBulkEmailRequest {
      from_email_address: Some("noreply@ranklab.gg".to_owned()),
      default_content: BulkEmailContent {
        template: Some(Template {
          template_name: Some("notification".to_owned()),
          template_data: Some(
            json!({
              "subject": "New VODs are available",
              "title": "There are new VODs available for review!",
              "body": "Go to your dashboard to start analyzing them.",
              "cta" : "View Available VODs",
              "cta_url" : "https://ranklab.gg/dashboard",
            })
            .to_string(),
          ),
          ..Default::default()
        }),
      },
      bulk_email_entries: coaches
        .iter()
        .map(|coach| BulkEmailEntry {
          destination: Destination {
            to_addresses: Some(vec![coach.email.clone()]),
            ..Default::default()
          },
          replacement_email_content: Some(ReplacementEmailContent {
            replacement_template: Some(ReplacementTemplate {
              replacement_template_data: Some(
                json!({
                  "name": coach.name.clone(),
                })
                .to_string(),
              ),
            }),
          }),
          ..Default::default()
        })
        .collect(),
      ..Default::default()
    };

    client.send_bulk_email(email_request).await.unwrap();
  });

  Response::Success(review)
}
