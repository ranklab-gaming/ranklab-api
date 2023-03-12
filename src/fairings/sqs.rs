use crate::aws;
use crate::config::Config;
use crate::guards::DbConn;
use crate::queue_handlers::stripe::{Connect, Direct};
use crate::queue_handlers::{S3BucketHandler, ScheduledTasksHandler, StripeHandler};
use anyhow::anyhow;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{tokio, Orbit, Rocket};
use rusoto_core::{HttpClient, Region};
use rusoto_sqs::{DeleteMessageRequest, ReceiveMessageRequest, Sqs, SqsClient};

#[derive(thiserror::Error, Debug)]
pub enum QueueHandlerError {
  #[error(transparent)]
  Ignorable(anyhow::Error),
  #[error(transparent)]
  Unknown(#[from] anyhow::Error),
}

#[async_trait]
pub trait QueueHandler: Send + Sync {
  fn new(db_conn: DbConn, config: Config) -> Self;
  fn url(&self) -> String;
  async fn handle(
    &self,
    message: &rusoto_sqs::Message,
    profile: &rocket::figment::Profile,
  ) -> Result<(), QueueHandlerError>;
}

#[derive(Clone)]
pub struct SqsFairing;

impl SqsFairing {
  pub fn fairing() -> impl Fairing {
    Self
  }

  async fn init(&self, rocket: &Rocket<Orbit>) {
    let config = rocket.state::<Config>().unwrap().clone();

    self.start::<S3BucketHandler>(rocket).await;
    self.start::<StripeHandler<Connect>>(rocket).await;
    self.start::<StripeHandler<Direct>>(rocket).await;

    if config.scheduled_tasks_queue.is_some() {
      self.start::<ScheduledTasksHandler>(rocket).await;
    }
  }

  async fn start<T: QueueHandler>(&self, rocket: &Rocket<Orbit>) {
    let db_conn = DbConn::get_one(rocket).await.unwrap();
    let config = rocket.state::<Config>().unwrap().clone();
    let profile = rocket.config().profile.clone();
    let aws_access_key_id = config.aws_access_key_id.clone();
    let aws_secret_key = config.aws_secret_key.clone();
    let fairing = self.clone();

    tokio::spawn(async move {
      let handler = T::new(db_conn, config);
      let mut builder = hyper::Client::builder();

      builder.pool_max_idle_per_host(0);

      let client = SqsClient::new_with(
        HttpClient::from_builder(builder, hyper_tls::HttpsConnector::new()),
        aws::CredentialsProvider::new(aws_access_key_id, aws_secret_key),
        Region::EuWest2,
      );

      loop {
        match fairing.poll(&handler, &client, &profile).await {
          Ok(_) => (),
          Err(e) => {
            error!("[sqs] {}", e);
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
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    let receive_request = ReceiveMessageRequest {
      queue_url: handler.url(),
      wait_time_seconds: Some(20),
      ..Default::default()
    };

    let response = client.receive_message(receive_request).await?;

    if let Some(messages) = response.messages {
      for message in messages {
        match handler.handle(&message, profile).await {
          Err(QueueHandlerError::Ignorable(e)) => {
            if profile == rocket::config::Config::RELEASE_PROFILE {
              return Err(e);
            } else {
              return Ok(());
            }
          }
          Err(e) => return Err(e.into()),
          Ok(()) => (),
        };

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

impl From<diesel::result::Error> for QueueHandlerError {
  fn from(e: diesel::result::Error) -> Self {
    if e == diesel::result::Error::NotFound {
      QueueHandlerError::Ignorable(e.into())
    } else {
      anyhow::Error::from(e).into()
    }
  }
}
