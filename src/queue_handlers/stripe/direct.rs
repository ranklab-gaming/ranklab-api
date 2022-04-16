use super::StripeEventHandler;
use crate::clients::StripeClient;
use crate::config::Config;
use crate::data_types::ReviewState;
use crate::emails::{Email, Recipient};
use crate::fairings::sqs::QueueHandlerError;
use crate::guards::DbConn;
use crate::models::{Coach, Review, ReviewChangeset};
use crate::stripe::webhook_events::{
  EventObject, EventObjectExt, EventType, EventTypeExt, WebhookEvent,
};
use anyhow::anyhow;
use diesel::prelude::*;
use serde_json::json;
use stripe::Expandable;

pub struct Direct {
  config: Config,
  db_conn: DbConn,
  client: StripeClient,
}

impl Direct {
  async fn handle_order_completed(&self, webhook: WebhookEvent) -> Result<(), QueueHandlerError> {
    let order_id = match &webhook.data.object {
      EventObject::Ext(EventObjectExt::Order(order)) => order.id.clone(),
      _ => return Ok(()),
    };

    self
      .db_conn
      .run(move |conn| {
        diesel::update(Review::find_by_order_id(order_id))
          .set(ReviewChangeset::default().state(ReviewState::AwaitingReview))
          .execute(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;

    let coaches = self.db_conn.run(move |conn| Coach::all(conn)).await?;

    let email = Email::new(
      &self.config,
      "notification".to_owned(),
      json!({
          "subject": "New VODs are available",
          "title": "There are new VODs available for review!",
          "body": "Go to your dashboard to start analyzing them.",
          "cta" : "View Available VODs",
          "cta_url" : "https://ranklab.gg/dashboard"
      }),
      coaches
        .iter()
        .map(|coach| {
          Recipient::new(
            coach.email.clone(),
            json!({
              "name": coach.name.clone(),
            }),
          )
        })
        .collect(),
    );

    email.deliver();

    Ok(())
  }

  async fn handle_charge_refunded(&self, webhook: WebhookEvent) -> Result<(), QueueHandlerError> {
    let charge = match &webhook.data.object {
      EventObject::Other(stripe::EventObject::Charge(charge)) => charge,
      _ => return Ok(()),
    };

    if !charge.refunded {
      return Ok(());
    }

    let payment_intent_id = match &charge.payment_intent {
      Some(Expandable::Id(payment_intent_id)) => payment_intent_id,
      _ => return Err(anyhow!("No payment intent id found in charge").into()),
    };

    let payment_intent = stripe::PaymentIntent::retrieve(&self.client.0, &payment_intent_id, &[])
      .await
      .map_err(anyhow::Error::from)?;

    let order_id = payment_intent
      .metadata
      .get("order_id")
      .ok_or(anyhow!("No order id found in payment intent metadata"))?
      .clone();

    self
      .db_conn
      .run(move |conn| {
        diesel::update(Review::find_by_order_id(order_id))
          .set(ReviewChangeset::default().state(ReviewState::Refunded))
          .execute(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;

    Ok(())
  }
}

#[async_trait]
impl StripeEventHandler for Direct {
  fn new(db_conn: DbConn, config: Config, client: StripeClient) -> Self {
    Self {
      config,
      db_conn,
      client,
    }
  }

  fn url(&self) -> String {
    self.config.stripe_direct_webhooks_queue.clone()
  }

  fn secret(&self) -> String {
    self.config.stripe_direct_webhooks_secret.clone()
  }

  async fn handle_event(&self, webhook: WebhookEvent) -> Result<(), QueueHandlerError> {
    match webhook.event_type {
      EventType::Ext(EventTypeExt::OrderCompleted) => self.handle_order_completed(webhook).await,
      EventType::Other(stripe::EventType::ChargeRefunded) => {
        self.handle_charge_refunded(webhook).await
      }
      _ => Ok(()),
    }
  }
}
