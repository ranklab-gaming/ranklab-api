use super::StripeEventHandler;
use crate::{config::Config, guards::DbConn};

pub struct Direct {
  config: Config,
}

impl Direct {}

#[async_trait]
impl StripeEventHandler for Direct {
  fn new(_db_conn: DbConn, config: Config) -> Self {
    Self { config }
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
      _ => (),
    }

    Ok(())
  }
}
