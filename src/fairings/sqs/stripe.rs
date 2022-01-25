use crate::config::Config;
use crate::fairings::sqs::QueueHandler;
use crate::guards::DbConn;
use diesel::prelude::*;
use serde::Deserialize;

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

pub struct StripeHandler {
  db_conn: DbConn,
  config: Config,
}

#[async_trait]
impl QueueHandler for StripeHandler {
  fn new(db_conn: DbConn, config: Config) -> Self {
    Self { db_conn, config }
  }

  fn url(&self) -> String {
    self.config.stripe_webhooks_queue.clone()
  }

  async fn handle(&self, message: &rusoto_sqs::Message) {
    use crate::schema::coaches::dsl::*;

    let body = message.body.clone().unwrap();
    let message_body: SqsMessageBody = serde_json::from_str(&body).unwrap();
    let webhook = stripe::Webhook::construct_event(
      message_body.body.as_str(),
      message_body.headers.stripe_signature.as_str(),
      self.config.stripe_webhooks_secret.as_str(),
    );

    if let Ok(webhook) = webhook {
      if webhook.event_type == stripe::EventType::AccountUpdated {
        if let stripe::EventObject::Account(account) = webhook.data.object {
          if account.payouts_enabled.unwrap_or(false) {
            self
              .db_conn
              .run(move |conn| {
                let existing_coach = coaches.filter(stripe_account_id.eq(account.id.to_string()));

                diesel::update(existing_coach)
                  .set(can_review.eq(true))
                  .execute(conn)
                  .unwrap();
              })
              .await;
          }
        }
      }
    }
  }
}
