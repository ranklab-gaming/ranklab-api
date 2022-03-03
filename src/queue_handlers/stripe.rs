use crate::config::Config;
use crate::fairings::sqs::{QueueHandler, QueueHandlerOutcome};
use crate::guards::DbConn;
use serde::Deserialize;
mod connect;
mod direct;
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
  fn new(db_conn: DbConn, config: Config) -> Self;
  fn url(&self) -> String;
  fn secret(&self) -> String;

  async fn handle_event(
    &self,
    webhook: stripe::WebhookEvent,
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()>;
}

pub struct StripeHandler<T: StripeEventHandler> {
  handler: T,
}

#[async_trait]
impl<T: StripeEventHandler + Sync + Send> QueueHandler for StripeHandler<T> {
  fn new(db_conn: DbConn, config: Config) -> Self {
    Self {
      handler: T::new(db_conn, config),
    }
  }

  fn url(&self) -> String {
    self.handler.url()
  }

  async fn handle(
    &self,
    message: &rusoto_sqs::Message,
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<QueueHandlerOutcome> {
    let body = message
      .body
      .clone()
      .ok_or(anyhow::anyhow!("No body in message"))?;

    let message_body: SqsMessageBody = serde_json::from_str(&body)?;

    let webhook = stripe::Webhook::construct_event(
      message_body.body.as_str(),
      message_body.headers.stripe_signature.as_str(),
      self.handler.secret().as_str(),
    )?;

    let livemode = match serde_json::from_str::<serde_json::Value>(message_body.body.as_str()) {
      Ok(event) => match event["livemode"].as_bool() {
        Some(livemode) => livemode,
        None => return Err(anyhow::anyhow!("Livemode is not present").into()),
      },
      Err(_) => return Err(anyhow::anyhow!("Could not parse message body").into()),
    };

    if profile == rocket::Config::RELEASE_PROFILE && !livemode {
      return Err(anyhow::anyhow!("Received webhook in test mode").into());
    }

    match self.handler.handle_event(webhook, profile).await {
      Ok(_) => Ok(QueueHandlerOutcome::Success),
      Err(e) => Err(e),
    }
  }
}