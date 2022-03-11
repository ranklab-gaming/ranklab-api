use super::StripeEventHandler;
use crate::config::Config;
use crate::guards::DbConn;
use crate::stripe::webhook_events::{EventObject, EventType, WebhookEvent};
use diesel::prelude::*;

pub struct Connect {
  db_conn: DbConn,
  config: Config,
}

impl Connect {
  async fn handle_account_updated(
    &self,
    webhook: &WebhookEvent,
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    use crate::schema::coaches::dsl::*;

    let account = match &webhook.data.object {
      EventObject::Other(stripe::EventObject::Account(account)) => account,
      _ => return Ok(()),
    };

    let details_submitted = match &account.details_submitted {
      Some(details_submitted) => *details_submitted,
      None => false,
    };

    let payouts_enabled = match &account.payouts_enabled {
      Some(payouts_enabled) => *payouts_enabled,
      None => false,
    };

    let account_id = account.id.clone();

    let result = self
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        let existing_coach = coaches.filter(stripe_account_id.eq(account_id.to_string()));

        diesel::update(existing_coach)
          .set((
            stripe_payouts_enabled.eq(payouts_enabled),
            stripe_details_submitted.eq(details_submitted),
          ))
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

#[async_trait]
impl StripeEventHandler for Connect {
  fn new(db_conn: DbConn, config: Config) -> Self {
    Self { db_conn, config }
  }

  fn url(&self) -> String {
    self.config.stripe_connect_webhooks_queue.clone()
  }

  fn secret(&self) -> String {
    self.config.stripe_connect_webhooks_secret.clone()
  }

  async fn handle_event(
    &self,
    webhook: WebhookEvent,
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    match webhook.event_type {
      EventType::Other(stripe::EventType::AccountUpdated) => {
        self.handle_account_updated(&webhook, profile).await?
      }
      _ => (),
    }

    Ok(())
  }
}
