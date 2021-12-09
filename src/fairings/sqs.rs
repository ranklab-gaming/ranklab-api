use crate::aws;
use crate::config::Config;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{tokio, Orbit, Rocket};
use rusoto_core::HttpClient;
use rusoto_core::Region;
use rusoto_sqs::{DeleteMessageRequest, ReceiveMessageRequest, Sqs, SqsClient};

pub struct SqsFairing;

impl SqsFairing {
  pub fn fairing() -> impl Fairing {
    Self
  }

  fn init(&self, config: &Config) {
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
              println!(
                "Received message '{}' with id {}",
                message.body.clone().unwrap(),
                message.message_id.clone().unwrap()
              );

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
    self.init(rocket.state::<Config>().unwrap());
  }
}
