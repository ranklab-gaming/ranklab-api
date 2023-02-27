use super::StripeEventHandler;
use crate::aws;
use crate::clients::StripeClient;
use crate::config::Config;
use crate::data_types::ReviewState;
use crate::emails::{Email, Recipient};
use crate::fairings::sqs::QueueHandlerError;
use crate::guards::DbConn;
use crate::models::{Coach, Review, ReviewChangeset};
use anyhow::anyhow;
use diesel::prelude::*;
use rusoto_core::{HttpClient, Region};
use rusoto_stepfunctions::StepFunctionsClient;
use serde_json::json;
use stripe::{EventObject, EventType, Expandable, WebhookEvent};

pub struct Direct {
  config: Config,
  db_conn: DbConn,
  step_functions: StepFunctionsClient,
}

impl Direct {
  async fn handle_payment_intent_succeeded(
    &self,
    webhook: WebhookEvent,
  ) -> Result<(), QueueHandlerError> {
    let payment_intent_id = match &webhook.data.object {
      EventObject::PaymentIntent(payment_intent) => payment_intent.id.clone(),
      _ => return Ok(()),
    };

    let review: Review = self
      .db_conn
      .run(move |conn| {
        Review::find_by_payment_intent_id(&payment_intent_id).get_result::<Review>(conn)
      })
      .await?;

    let coach_id = review.coach_id;

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

    let coach: Coach = self
      .db_conn
      .run(move |conn| Coach::find_by_id(&coach_id).first(conn))
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
          "name": coach.name,
        }),
      )],
    );

    email.deliver();

    Ok(())
  }

  async fn handle_charge_refunded(&self, webhook: WebhookEvent) -> Result<(), QueueHandlerError> {
    let charge = match webhook.data.object {
      EventObject::Charge(charge) => charge,
      _ => return Ok(()),
    };

    if !charge.refunded {
      return Ok(());
    }

    let payment_intent_id = match charge.payment_intent {
      Some(Expandable::Id(payment_intent_id)) => payment_intent_id,
      _ => return Err(anyhow!("No payment intent id found in charge").into()),
    };

    self
      .db_conn
      .run(move |conn| {
        diesel::update(Review::find_by_payment_intent_id(&payment_intent_id))
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
  fn new(db_conn: DbConn, config: Config, _client: StripeClient) -> Self {
    let aws_access_key_id = config.aws_access_key_id.clone();
    let aws_secret_key = config.aws_secret_key.clone();
    let mut builder = hyper::Client::builder();

    builder.pool_max_idle_per_host(0);

    Self {
      config,
      db_conn,
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
      EventType::PaymentIntentSucceeded => self.handle_payment_intent_succeeded(webhook).await,
      EventType::ChargeRefunded => self.handle_charge_refunded(webhook).await,
      _ => Ok(()),
    }
  }
}
