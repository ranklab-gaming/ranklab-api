use super::StripeEventHandler;
use crate::{clients::StripeClient, config::Config, guards::DbConn};
use diesel::prelude::*;
use stripe::Expandable;

pub struct Direct {
  db_conn: DbConn,
  config: Config,
}

impl Direct {
  async fn handle_checkout_session_completed(
    &self,
    webhook: &stripe::WebhookEvent,
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    use crate::schema::players::dsl::*;

    let stripe_client = StripeClient::new(&self.config).0;

    let checkout_session = match webhook.data.object.clone() {
      stripe::EventObject::CheckoutSession(checkout_session) => checkout_session,
      _ => return Ok(()),
    };

    let setup_intent_id = match checkout_session.setup_intent.clone() {
      Some(box Expandable::Id(setup_intent_id)) => setup_intent_id,
      Some(box Expandable::Object(setup_intent)) => setup_intent.id,
      None => return Err(anyhow::anyhow!("No setup intent")),
    };

    let setup_intent = stripe::SetupIntent::retrieve(&stripe_client, &setup_intent_id, &[]).await?;

    let payment_method_id = match setup_intent.payment_method {
      Some(box Expandable::Id(payment_method_id)) => payment_method_id,
      Some(box Expandable::Object(payment_method)) => payment_method.id,
      None => return Err(anyhow::anyhow!("No payment method")),
    };

    let customer_id = match checkout_session.customer {
      Some(box Expandable::Id(customer_id)) => customer_id,
      Some(box Expandable::Object(customer)) => customer.id,
      None => return Err(anyhow::anyhow!("No customer")),
    };

    stripe::Customer::update(
      &stripe_client,
      &customer_id,
      stripe::UpdateCustomer {
        invoice_settings: Some(
          stripe::CustomerInvoiceSettings {
            default_payment_method: Some(payment_method_id.to_string().into()),
            footer: None,
            custom_fields: None,
          }
          .into(),
        ),
        ..Default::default()
      },
    )
    .await
    .map_err(|e| anyhow::anyhow!("{}", e))?;

    let result = self
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        let existing_player = players.filter(stripe_customer_id.eq(customer_id.to_string()));

        diesel::update(existing_player)
          .set(can_create_reviews.eq(true))
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

  async fn handle_customer_updated(
    &self,
    webhook: &stripe::WebhookEvent,
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    use crate::schema::players::dsl::*;

    let customer = match webhook.data.object.clone() {
      stripe::EventObject::Customer(customer) => customer,
      _ => return Ok(()),
    };

    let invoice_settings = match customer.clone().invoice_settings {
      Some(box invoice_settings) => invoice_settings,
      None => return Err(anyhow::anyhow!("No invoice settings")),
    };

    let default_payment_method_id = match invoice_settings.default_payment_method {
      Some(box Expandable::Id(default_payment_method_id)) => Some(default_payment_method_id),
      Some(box Expandable::Object(default_payment_method)) => Some(default_payment_method.id),
      None => None,
    };

    let result = self
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        let existing_player = players.filter(stripe_customer_id.eq(customer.id.to_string()));

        diesel::update(existing_player)
          .set(can_create_reviews.eq(default_payment_method_id.is_some()))
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
impl StripeEventHandler for Direct {
  fn new(db_conn: DbConn, config: Config) -> Self {
    Self { db_conn, config }
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
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    match webhook.event_type {
      stripe::EventType::CheckoutSessionCompleted => {
        self
          .handle_checkout_session_completed(&webhook, profile)
          .await?
      }
      stripe::EventType::CustomerUpdated => self.handle_customer_updated(&webhook, profile).await?,
      _ => (),
    }

    Ok(())
  }
}
