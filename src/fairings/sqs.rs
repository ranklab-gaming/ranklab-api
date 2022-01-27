use crate::aws;
use crate::config::Config;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{tokio, Orbit, Rocket};
use rusoto_core::HttpClient;
use rusoto_core::Region;
mod s3_bucket;
mod stripe;
use self::stripe::StripeHandler;
use crate::guards::DbConn;
use rocket::futures::FutureExt;
use rusoto_sqs::{DeleteMessageRequest, ReceiveMessageRequest, Sqs, SqsClient};
use s3_bucket::S3BucketHandler;
use std::panic::AssertUnwindSafe;

pub struct SqsFairing;

#[async_trait]
pub trait QueueHandler: Send + Sync {
  fn new(db_conn: DbConn, config: Config) -> Self;
  fn url(&self) -> String;
  async fn handle(&self, message: &rusoto_sqs::Message) -> ();
}

impl SqsFairing {
  pub fn fairing() -> impl Fairing {
    Self
  }

  async fn init(rocket: &Rocket<Orbit>) {
    Self::start::<S3BucketHandler>(rocket).await;
    Self::start::<StripeHandler>(rocket).await;
  }

  async fn start<T: QueueHandler>(rocket: &Rocket<Orbit>) {
    let db_conn = DbConn::get_one(&rocket)
      .await
      .expect("Failed to get db connection");

    let config = rocket.state::<Config>().unwrap().clone();
    let aws_access_key_id = config.aws_access_key_id.clone();
    let aws_secret_key = config.aws_secret_key.clone();

    tokio::spawn(async move {
      let handler = T::new(db_conn, config);

      let client = SqsClient::new_with(
        HttpClient::new().unwrap(),
        aws::CredentialsProvider::new(aws_access_key_id, aws_secret_key),
        Region::EuWest2,
      );

      loop {
        let _ = AssertUnwindSafe(Self::poll(&handler, &client))
          .catch_unwind()
          .await;
      }
    });
  }

  async fn poll<T: QueueHandler>(handler: &T, client: &SqsClient) {
    let receive_request = ReceiveMessageRequest {
      queue_url: handler.url(),
      wait_time_seconds: Some(20),
      ..Default::default()
    };

    let response = client.receive_message(receive_request).await.unwrap();

    if let Some(messages) = response.messages {
      for message in messages {
        handler.handle(&message).await;

        let delete_request = DeleteMessageRequest {
          queue_url: handler.url(),
          receipt_handle: message.receipt_handle.clone().unwrap(),
        };

        client.delete_message(delete_request).await.unwrap();
      }
    }
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
    Self::init(rocket).await;
  }
}
