use crate::clients::StripeClient;
use crate::config::Config;
use crate::fairings::sqs::{QueueHandler, QueueHandlerOutcome};
use crate::guards::DbConn;
use diesel::prelude::*;
use serde::Deserialize;
use stripe::Expandable;

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

pub struct Direct {
  db_conn: DbConn,
  config: Config,
}

pub struct Connect {
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

    let result = self
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        let existing_player = players.filter(stripe_customer_id.eq(customer_id.to_string()));

        diesel::update(existing_player)
          .set(stripe_payment_method_id.eq(payment_method_id.to_string()))
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

impl Connect {
  async fn handle_account_updated(
    &self,
    webhook: &stripe::WebhookEvent,
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    use crate::schema::coaches::dsl::*;

    let account = match &webhook.data.object {
      stripe::EventObject::Account(account) => account,
      _ => return Ok(()),
    };

    let details_submitted = match &account.details_submitted {
      Some(details_submitted) => **details_submitted,
      None => false,
    };

    let payouts_enabled = match &account.payouts_enabled {
      Some(payouts_enabled) => **payouts_enabled,
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
      _ => (),
    }

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
    webhook: stripe::WebhookEvent,
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    match webhook.event_type {
      stripe::EventType::AccountUpdated => self.handle_account_updated(&webhook, profile).await?,
      _ => (),
    }

    Ok(())
  }
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
