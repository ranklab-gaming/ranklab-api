use crate::clients::StripeClient;
use crate::config::Config;
use crate::guards::DbConn;
use crate::queue_handlers::QueueHandler;
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

    match webhook.event_type {
      stripe::EventType::AccountUpdated => self.handle_account_updated(&webhook, profile).await,
      stripe::EventType::CheckoutSessionCompleted => {
        self
          .handle_checkout_session_completed(&webhook, profile)
          .await
      }
      _ => Ok(()),
    }
  }
}

impl StripeHandler {
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
