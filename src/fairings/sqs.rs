use crate::aws;
use crate::config::Config;
use crate::db::DbConn;
use diesel::prelude::*;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{tokio, Orbit, Rocket};
use rusoto_core::HttpClient;
use rusoto_core::Region;
use rusoto_sqs::{DeleteMessageRequest, ReceiveMessageRequest, Sqs, SqsClient};
use serde::Deserialize;

pub struct SqsFairing;

#[derive(Deserialize)]
struct RecordS3Object {
  key: String,
}

#[derive(Deserialize)]
struct RecordS3 {
  object: RecordS3Object,
}

#[derive(Deserialize)]
struct Record {
  s3: RecordS3,
}

#[derive(Deserialize)]
struct SqsMessageBody {
  #[serde(rename = "Records")]
  records: Vec<Record>,
}

impl SqsFairing {
  pub fn fairing() -> impl Fairing {
    Self
  }

  async fn init(&self, rocket: &Rocket<Orbit>) {
    let db_conn = DbConn::get_one(&rocket)
      .await
      .expect("Failed to get db connection");

    let config = rocket.state::<Config>().unwrap();
    let queue_url = config.s3_bucket_queue.clone();
    let aws_access_key_id = config.aws_access_key_id.clone();
    let aws_secret_key = config.aws_secret_key.clone();

    tokio::spawn(async move {
      let client = SqsClient::new_with(
        HttpClient::new().unwrap(),
        aws::CredentialsProvider::new(aws_access_key_id, aws_secret_key),
        Region::EuWest2,
      );

      loop {
        let receive_request = ReceiveMessageRequest {
          queue_url: queue_url.clone(),
          wait_time_seconds: Some(20),
          ..Default::default()
        };

        let response = client.receive_message(receive_request).await.unwrap();

        match response.messages {
          Some(messages) => {
            for message in messages {
              use crate::schema::recordings;
              use crate::schema::recordings::dsl::*;

              let body = message.body.unwrap();
              let message_body: SqsMessageBody = serde_json::from_str(&body).unwrap();

              for record in message_body.records {
                db_conn
                  .run(move |conn| {
                    let existing_recording =
                      recordings::table.filter(recordings::video_key.eq(&record.s3.object.key));

                    diesel::update(existing_recording)
                      .set(uploaded.eq(true))
                      .execute(conn)
                      .unwrap();
                  })
                  .await;
              }

              let delete_request = DeleteMessageRequest {
                queue_url: queue_url.clone(),
                receipt_handle: message.receipt_handle.clone().unwrap(),
              };

              client.delete_message(delete_request).await.unwrap();
            }
          }
          None => {}
        }
      }
    });
  }
}

#[rocket::async_trait]
impl Fairing for SqsFairing {
  fn info(&self) -> Info {
    Info {
      name: "sqs",
      kind: Kind::Liftoff,
    }
  }

  async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
    self.init(rocket).await;
  }
}
