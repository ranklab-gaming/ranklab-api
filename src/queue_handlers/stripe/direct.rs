use super::StripeEventHandler;
use crate::data_types::ReviewState;
use crate::emails::{Email, Recipient};
use crate::models::{Coach, Review};
use crate::{config::Config, guards::DbConn};
use diesel::prelude::*;
use serde_json::json;
use stripe::Expandable;

pub struct Direct {
  config: Config,
  db_conn: DbConn,
}

impl Direct {
  async fn handle_payment_intent_succeeded(
    &self,
    webhook: stripe::WebhookEvent,
  ) -> anyhow::Result<()> {
    let payment_intent_id = match &webhook.data.object {
      stripe::EventObject::PaymentIntent(payment_intent) => payment_intent.id.clone(),
      _ => return Ok(()),
    };

    self
      .db_conn
      .run(move |conn| {
        use crate::schema::reviews::dsl::{reviews, state, stripe_payment_intent_id};

        diesel::update(reviews.filter(stripe_payment_intent_id.eq(payment_intent_id.to_string())))
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

  async fn handle_charge_refunded(&self, webhook: stripe::WebhookEvent) -> anyhow::Result<()> {
    let charge = match &webhook.data.object {
      stripe::EventObject::Charge(charge) => charge,
      _ => return Ok(()),
    };

    if !charge.refunded {
      return Ok(());
    }

    let payment_intent_id = match charge.payment_intent.clone() {
      Some(Expandable::Id(payment_intent_id)) => payment_intent_id,
      Some(Expandable::Object(payment_intent)) => payment_intent.id,
      None => return Err(anyhow::anyhow!("No payment intent found")),
    };

    self
      .db_conn
      .run(move |conn| {
        use crate::schema::reviews::dsl::{reviews, state, stripe_payment_intent_id};

        diesel::update(reviews.filter(stripe_payment_intent_id.eq(payment_intent_id.to_string())))
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
  fn new(db_conn: DbConn, config: Config) -> Self {
    Self { config, db_conn }
  }

  fn url(&self) -> String {
    self.config.stripe_direct_webhooks_queue.clone()
  }

  fn secret(&self) -> String {
    self.config.stripe_direct_webhooks_secret.clone()
  }

  async fn handle_event(
    &self,
    webhook: stripe::WebhookEvent,
    _profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    match webhook.event_type {
      stripe::EventType::PaymentIntentSucceeded => {
        self.handle_payment_intent_succeeded(webhook).await?
      }
      stripe::EventType::ChargeRefunded => self.handle_charge_refunded(webhook).await?,
      _ => (),
    }

    Ok(())
  }
}
