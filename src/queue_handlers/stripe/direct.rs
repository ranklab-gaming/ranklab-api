use super::StripeEventHandler;
use crate::aws;
use crate::clients::StripeClient;
use crate::config::Config;
use crate::data_types::ReviewState;
use crate::emails::{Email, Recipient};
use crate::fairings::sqs::QueueHandlerError;
use crate::guards::DbConn;
use crate::models::{Coach, Review, ReviewChangeset};
use crate::schema::coaches;
use anyhow::anyhow;
use diesel::prelude::*;
use rusoto_core::{HttpClient, Region};
use rusoto_stepfunctions::StepFunctionsClient;
use serde_json::json;
use stripe::{EventObject, EventType, Expandable, WebhookEvent};

pub struct Direct {
  config: Config,
  db_conn: DbConn,
  client: StripeClient,
  step_functions: StepFunctionsClient,
}

impl Direct {
  async fn handle_order_completed(&self, webhook: WebhookEvent) -> Result<(), QueueHandlerError> {
    let order_id = match &webhook.data.object {
      EventObject::Order(order) => order.id.clone(),
      _ => return Ok(()),
    };

    let review: Review = self
      .db_conn
      .run(move |conn| Review::find_by_order_id(&order_id).get_result::<Review>(conn))
      .await?;

    let coach_id = review.coach_id.clone();

    if let Some(state_machine_arn) = &self.config.scheduled_tasks_state_machine_arn {
      rusoto_stepfunctions::StepFunctions::start_execution(
        &self.step_functions,
        rusoto_stepfunctions::StartExecutionInput {
          state_machine_arn: state_machine_arn.clone(),
          input: Some(
            serde_json::json!({ "input": { "reviewId": review.id.to_string() } }).to_string(),
          ),
          name: None,
          trace_header: None,
        },
      )
      .await
      .map_err(anyhow::Error::from)?;
    }

    self
      .db_conn
      .run(move |conn| {
        diesel::update(&review)
          .set(ReviewChangeset::default().state(ReviewState::AwaitingReview))
          .execute(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;

    // only email the coach if he has been specifcally requested
    if let Some(coach_id) = coach_id {
      let coach: Coach = self
        .db_conn
        .run(move |conn| coaches::table.find(coach_id).first(conn))
        .await?;

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
        vec![Recipient::new(
          coach.email.clone(),
          json!({
            "name": coach.name.clone(),
          }),
        )],
      );

      email.deliver();
    }

    Ok(())
  }

  async fn handle_charge_refunded(&self, webhook: WebhookEvent) -> Result<(), QueueHandlerError> {
    let charge = match &webhook.data.object {
      EventObject::Charge(charge) => charge,
      _ => return Ok(()),
    };

    if !charge.refunded {
      return Ok(());
    }

    let payment_intent_id = match &charge.payment_intent {
      Some(Expandable::Id(payment_intent_id)) => payment_intent_id,
      _ => return Err(anyhow!("No payment intent id found in charge").into()),
    };

    let payment_intent = stripe::PaymentIntent::retrieve(&self.client.0, &payment_intent_id, &[])
      .await
      .map_err(anyhow::Error::from)?;

    let order_id = payment_intent
      .metadata
      .get("order_id")
      .ok_or(anyhow!("No order id found in payment intent metadata"))?
      .clone();

    self
      .db_conn
      .run(move |conn| {
        diesel::update(Review::find_by_order_id(&order_id))
          .set(ReviewChangeset::default().state(ReviewState::Refunded))
          .execute(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;

    Ok(())
  }
}

#[async_trait]
impl StripeEventHandler for Direct {
  fn new(db_conn: DbConn, config: Config, client: StripeClient) -> Self {
    let aws_access_key_id = config.aws_access_key_id.clone();
    let aws_secret_key = config.aws_secret_key.clone();
    let mut builder = hyper::Client::builder();

    builder.pool_max_idle_per_host(0);

    Self {
      config,
      db_conn,
      client,
      step_functions: StepFunctionsClient::new_with(
        HttpClient::from_builder(builder, hyper_tls::HttpsConnector::new()),
        aws::CredentialsProvider::new(aws_access_key_id, aws_secret_key),
        Region::EuWest2,
      ),
    }
  }

  fn url(&self) -> String {
    self.config.stripe_direct_webhooks_queue.clone()
  }

  fn secret(&self) -> String {
    self.config.stripe_direct_webhooks_secret.clone()
  }

  async fn handle_event(&self, webhook: WebhookEvent) -> Result<(), QueueHandlerError> {
    match webhook.event_type {
      EventType::OrderCompleted => self.handle_order_completed(webhook).await,
      EventType::ChargeRefunded => self.handle_charge_refunded(webhook).await,
      _ => Ok(()),
    }
  }
}
