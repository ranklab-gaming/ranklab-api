use crate::config::Config;
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use serde::Deserialize;
use stripe::{Webhook, WebhookEvent};
mod connect;
mod direct;
use crate::clients::StripeClient;
use anyhow::anyhow;
pub use connect::Connect;
pub use direct::Direct;

#[derive(Deserialize)]
struct Headers {
  #[serde(rename = "Stripe-Signature")]
  stripe_signature: String,
}

#[derive(Deserialize)]
struct SqsMessageBody {
  body: String,
  headers: Headers,
}

#[async_trait]
pub trait StripeEventHandler {
  fn new(db_conn: DbConn, config: Config, client: StripeClient) -> Self;
  fn url(&self) -> String;
  fn secret(&self) -> String;
  async fn handle_event(&self, webhook: WebhookEvent) -> Result<(), QueueHandlerError>;
}

pub struct StripeHandler<T: StripeEventHandler> {
  handler: T,
}

#[async_trait]
impl<T: StripeEventHandler + Sync + Send> QueueHandler for StripeHandler<T> {
  fn new(db_conn: DbConn, config: Config) -> Self {
    let client = StripeClient::new(&config);

    Self {
      handler: T::new(db_conn, config, client),
    }
  }

  fn url(&self) -> String {
    self.handler.url()
  }

  async fn handle(&self, message: &rusoto_sqs::Message) -> Result<(), QueueHandlerError> {
    let body = message
      .body
      .clone()
      .ok_or(anyhow!("No body found in sqs message"))?;

    let message_body: SqsMessageBody = serde_json::from_str(&body).map_err(anyhow::Error::from)?;

    let webhook = Webhook::construct_event(
      message_body.body.as_str(),
      message_body.headers.stripe_signature.as_str(),
      self.handler.secret().as_str(),
    )
    .map_err(anyhow::Error::from)?;

    if !webhook.livemode {
      return Err(QueueHandlerError::Ignorable(anyhow!(
        "Received webhook in test mode"
      )));
    }

    self.handler.handle_event(webhook).await
  }
}
