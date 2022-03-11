use super::StripeEventHandler;
use crate::clients::StripeClient;
use crate::config::Config;
use crate::data_types::ReviewState;
use crate::emails::{Email, Recipient};
use crate::guards::DbConn;
use crate::models::{Coach, Review};
use crate::stripe::webhook_events::{
  EventObject, EventObjectExt, EventType, EventTypeExt, WebhookEvent,
};
use diesel::prelude::*;
use serde_json::json;
use stripe::Expandable;

pub struct Direct {
  config: Config,
  db_conn: DbConn,
  client: StripeClient,
}

impl Direct {
  async fn handle_order_completed(&self, webhook: WebhookEvent) -> anyhow::Result<()> {
    let order_id = match &webhook.data.object {
      EventObject::Ext(EventObjectExt::Order(order)) => order.id.clone(),
      _ => return Ok(()),
    };

    self
      .db_conn
      .run(move |conn| {
        use crate::schema::reviews::dsl::{reviews, state, stripe_order_id};

        diesel::update(reviews.filter(stripe_order_id.eq(order_id.to_string())))
          .set(state.eq(ReviewState::AwaitingReview))
          .get_result::<Review>(conn)
          .unwrap()
      })
      .await;

    let coaches = self
      .db_conn
      .run(move |conn| {
        use crate::schema::coaches::dsl::*;

        coaches.load::<Coach>(conn).unwrap()
      })
      .await;

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

  async fn handle_charge_refunded(&self, webhook: WebhookEvent) -> anyhow::Result<()> {
    let charge = match &webhook.data.object {
      EventObject::Other(stripe::EventObject::Charge(charge)) => charge,
      _ => return Ok(()),
    };

    if !charge.refunded {
      return Ok(());
    }

    let payment_intent_id = match &charge.payment_intent {
      Some(Expandable::Id(payment_intent_id)) => payment_intent_id,
      _ => return Err(anyhow::anyhow!("No payment intent id found")),
    };

    let payment_intent =
      stripe::PaymentIntent::retrieve(&self.client.0, &payment_intent_id, &[]).await?;

    let order_id = payment_intent
      .metadata
      .get("order_id")
      .ok_or(anyhow::anyhow!(
        "No order id found in payment intent metadata"
      ))?
      .clone();

    self
      .db_conn
      .run(move |conn| {
        use crate::schema::reviews::dsl::{reviews, state, stripe_order_id};

        diesel::update(reviews.filter(stripe_order_id.eq(order_id)))
          .set(state.eq(ReviewState::Refunded))
          .get_result::<Review>(conn)
          .unwrap()
      })
      .await;

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

  async fn handle_event(
    &self,
    webhook: WebhookEvent,
    _profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    match webhook.event_type {
      EventType::Ext(EventTypeExt::OrderCompleted) => self.handle_order_completed(webhook).await?,
      EventType::Other(stripe::EventType::ChargeRefunded) => {
        self.handle_charge_refunded(webhook).await?
      }
      _ => (),
    }

    Ok(())
  }
}
