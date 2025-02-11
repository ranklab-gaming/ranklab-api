use crate::aws::ConfigCredentialsProvider;
use crate::config::Config;
use crate::guards::DbConn;
use crate::queue_handlers::{RekognitionHandler, UploadsHandler};
use anyhow::{anyhow, Result};
use hyper_tls::HttpsConnector;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{tokio, Orbit, Rocket};
use rusoto_core::{HttpClient, Region, RusotoError};
use rusoto_sqs::{
  ChangeMessageVisibilityError, ChangeMessageVisibilityRequest, DeleteMessageRequest,
  ReceiveMessageRequest, Sqs, SqsClient,
};
use tokio::time::{sleep, Duration};

#[async_trait]
pub trait QueueHandler: Send + Sync {
  fn new(db_conn: DbConn, config: Config) -> Self;
  fn url(&self) -> String;
  fn name(&self) -> &'static str;
  async fn instance_id(&self, message: String) -> Result<Option<String>>;
  async fn handle(&self, message: String) -> Result<()>;
}

#[derive(Clone)]
pub struct SqsFairing;

impl SqsFairing {
  pub fn fairing() -> impl Fairing {
    Self
  }

  async fn init(&self, rocket: &Rocket<Orbit>) {
    self.start::<UploadsHandler>(rocket).await;
    self.start::<RekognitionHandler>(rocket).await;
  }

  async fn start<T: QueueHandler>(&self, rocket: &Rocket<Orbit>) {
    let db_conn = DbConn::get_one(rocket).await.unwrap();
    let config = rocket.state::<Config>().unwrap().clone();
    let instance_id = config.instance_id.clone();
    let fairing = self.clone();

    tokio::spawn(async move {
      let handler = T::new(db_conn, config.clone());

      let client = SqsClient::new_with(
        HttpClient::from_connector(HttpsConnector::new()),
        ConfigCredentialsProvider::new(config),
        Region::EuWest2,
      );

      loop {
        match fairing.poll(&handler, &client, &instance_id).await {
          Ok(should_delay_poll) => {
            if should_delay_poll {
              sleep(Duration::from_secs(5)).await;
            }
          }
          Err(e) => {
            error!("[sqs] [{}] {:?}", handler.name(), e);
            sentry::capture_error(e.root_cause());
          }
        }
      }
    });
  }

  async fn poll<T: QueueHandler>(
    &self,
    handler: &T,
    client: &SqsClient,
    instance_id: &Option<String>,
  ) -> anyhow::Result<bool> {
    let receive_request = ReceiveMessageRequest {
      queue_url: handler.url(),
      wait_time_seconds: Some(20),
      ..Default::default()
    };

    let response = client.receive_message(receive_request).await?;
    let mut should_delay_poll = false;

    if let Some(messages) = response.messages {
      for message in messages {
        info!(
          "[sqs] [{}] Received message: {:?}",
          handler.name(),
          message.message_id
        );

        let body = message
          .body
          .clone()
          .ok_or_else(|| anyhow!("No body found in sqs message"))?;

        info!("[sqs] [{}] Message body: {:?}", handler.name(), body);

        let message_instance_id = handler.instance_id(body.clone()).await?;

        if message_instance_id != *instance_id {
          info!(
            "[sqs] [{}] Message {:?} is for instance {:?}, skipping",
            handler.name(),
            message.message_id,
            message_instance_id
          );

          self
            .change_message_visibility(
              client,
              &handler.url(),
              &message
                .receipt_handle
                .clone()
                .ok_or_else(|| anyhow!("Receipt handle missing from message"))?,
              0,
            )
            .await?;

          should_delay_poll = true;

          continue;
        }

        handler.handle(body).await?;

        info!(
          "[sqs] [{}] Deleting message: {:?}",
          handler.name(),
          message.message_id
        );

        let delete_request = DeleteMessageRequest {
          queue_url: handler.url(),
          receipt_handle: message
            .receipt_handle
            .clone()
            .ok_or_else(|| anyhow!("Receipt handle missing from message"))?,
        };

        client.delete_message(delete_request).await?;
      }
    };

    Ok(should_delay_poll)
  }

  async fn change_message_visibility(
    &self,
    sqs_client: &SqsClient,
    queue_url: &str,
    receipt_handle: &str,
    visibility_timeout: i64,
  ) -> Result<(), RusotoError<ChangeMessageVisibilityError>> {
    let req = ChangeMessageVisibilityRequest {
      queue_url: queue_url.to_owned(),
      receipt_handle: receipt_handle.to_owned(),
      visibility_timeout,
    };

    sqs_client.change_message_visibility(req).await?;

    Ok(())
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
