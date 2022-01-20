use crate::aws;
use crate::config::Config;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{tokio, Orbit, Rocket};
use rusoto_core::HttpClient;
use rusoto_core::Region;
mod s3_bucket;
use crate::guards::DbConn;
use rusoto_sqs::{DeleteMessageRequest, ReceiveMessageRequest, Sqs, SqsClient};
use s3_bucket::S3BucketHandler;

pub struct SqsFairing;

#[async_trait]
pub trait QueueHandler: Send + Sync {
  fn new(db_conn: DbConn) -> Self;
  fn url(config: &Config) -> String;
  async fn handle(&self, message: &rusoto_sqs::Message) -> ();
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

    self.poll::<S3BucketHandler>(config, db_conn);
  }

  fn poll<T: QueueHandler>(&self, config: &Config, db_conn: DbConn) {
    let queue_url = T::url(&config);
    let aws_access_key_id = config.aws_access_key_id.clone();
    let aws_secret_key = config.aws_secret_key.clone();

    tokio::spawn(async move {
      let handler = T::new(db_conn);

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
              handler.handle(&message).await;

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
