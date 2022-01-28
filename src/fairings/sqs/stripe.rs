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

  async fn handle(
    &self,
    message: &rusoto_sqs::Message,
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    use crate::schema::coaches::dsl::*;

    let body = message
      .body
      .clone()
      .ok_or(anyhow::anyhow!("No body in message"))?;

    let message_body: SqsMessageBody = serde_json::from_str(&body)?;

    let webhook = stripe::Webhook::construct_event(
      message_body.body.as_str(),
      message_body.headers.stripe_signature.as_str(),
      self.config.stripe_webhooks_secret.as_str(),
    )?;

    if webhook.event_type != stripe::EventType::AccountUpdated {
      return Ok(());
    };

    let account = match webhook.data.object {
      stripe::EventObject::Account(account) => account,
      _ => return Ok(()),
    };

    let payouts_enabled = match account.payouts_enabled {
      Some(payouts_enabled) => *payouts_enabled,
      None => false,
    };

    if !payouts_enabled {
      return Ok(());
    }

    let account_id = account.id.clone();

    let result = self
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        let existing_coach = coaches.filter(stripe_account_id.eq(account_id.to_string()));

        diesel::update(existing_coach)
          .set(can_review.eq(true))
          .execute(conn)?;

        Ok(())
      })
      .await;

    if let Err(diesel::result::Error::NotFound) = result {
      if profile == rocket::Config::RELEASE_PROFILE {
        return Err(diesel::result::Error::NotFound.into());
      } else {
        return Ok(());
      }
    }

    result?;
    Ok(())
  }
}
