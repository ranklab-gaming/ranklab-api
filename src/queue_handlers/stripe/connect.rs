use super::StripeEventHandler;
use crate::clients::StripeClient;
use crate::config::Config;
use crate::fairings::sqs::QueueHandlerError;
use crate::guards::DbConn;
use crate::models::{Coach, CoachChangeset};
use crate::stripe::webhook_events::{EventObject, EventType, WebhookEvent};
use diesel::prelude::*;

pub struct Connect {
  db_conn: DbConn,
  config: Config,
}

impl Connect {
  async fn handle_account_updated(&self, webhook: &WebhookEvent) -> Result<(), QueueHandlerError> {
    let account = match &webhook.data.object {
      EventObject::Other(stripe::EventObject::Account(account)) => account,
      _ => return Ok(()),
    };

    let details_submitted = account.details_submitted.unwrap_or(false);
    let payouts_enabled = account.payouts_enabled.unwrap_or(false);
    let account_id = account.id.clone();

    self
      .db_conn
      .run(move |conn| {
        diesel::update(Coach::find_by_stripe_account_id(account_id))
          .set(
            CoachChangeset::default()
              .stripe_details_submitted(details_submitted)
              .stripe_payouts_enabled(payouts_enabled),
          )
          .execute(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;

    Ok(())
  }
}

#[async_trait]
impl StripeEventHandler for Connect {
  fn new(db_conn: DbConn, config: Config, _client: StripeClient) -> Self {
    Self { db_conn, config }
  }

  fn url(&self) -> String {
    self.config.stripe_connect_webhooks_queue.clone()
  }

  fn secret(&self) -> String {
    self.config.stripe_connect_webhooks_secret.clone()
  }

  async fn handle_event(&self, webhook: WebhookEvent) -> Result<(), QueueHandlerError> {
    match webhook.event_type {
      EventType::Other(stripe::EventType::AccountUpdated) => {
        self.handle_account_updated(&webhook).await
      }
      _ => Ok(()),
    }
  }
}
