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
  async fn handle_event(
    &self,
    webhook: WebhookEvent,
    profile: &rocket::figment::Profile,
  ) -> Result<(), QueueHandlerError>;
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

  async fn instance_id(
    &self,
    message: &rusoto_sqs::Message,
    profile: &rocket::figment::Profile,
  ) -> Result<Option<String>, QueueHandlerError> {
    let webhook = self.parse_webhook(message, profile)?;

    let metadata = match webhook.data.object {
      stripe::EventObject::Account(account) => Some(account.metadata),
      stripe::EventObject::Charge(charge) => Some(charge.metadata),
      stripe::EventObject::PaymentIntent(payment_intent) => Some(payment_intent.metadata),
      _ => None,
    };

    Ok(metadata.and_then(|metadata| metadata.get("instance_id").map(|s| s.to_string())))
  }

  async fn handle(
    &self,
    message: &rusoto_sqs::Message,
    profile: &rocket::figment::Profile,
  ) -> Result<(), QueueHandlerError> {
    let webhook = self.parse_webhook(message, profile)?;
    self.handler.handle_event(webhook, profile).await
  }
}

impl<T: StripeEventHandler> StripeHandler<T> {
  fn parse_webhook(
    &self,
    message: &rusoto_sqs::Message,
    profile: &rocket::figment::Profile,
  ) -> Result<WebhookEvent, QueueHandlerError> {
    let body = message
      .body
      .clone()
      .ok_or_else(|| anyhow!("No body found in sqs message"))?;

    let message_body: SqsMessageBody = serde_json::from_str(&body).map_err(anyhow::Error::from)?;

    let webhook = Webhook::construct_event(
      message_body.body.as_str(),
      message_body.headers.stripe_signature.as_str(),
      self.handler.secret().as_str(),
    )
    .map_err(anyhow::Error::from)?;

    if profile == rocket::Config::RELEASE_PROFILE && !webhook.livemode {
      return Err(QueueHandlerError::Ignorable(anyhow!(
        "Received webhook in test mode"
      )));
    }

    Ok(webhook)
  }
}
