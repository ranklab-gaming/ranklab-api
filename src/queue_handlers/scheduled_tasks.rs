use crate::config::Config;
use crate::data_types::ReviewState;
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use crate::models::{Review, ReviewChangeset};
use crate::schema::reviews;
use anyhow::anyhow;
use diesel::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct SqsMessageBody {
  #[serde(rename = "reviewId")]
  review_id: uuid::Uuid,
}

pub struct ScheduledTasksHandler {
  db_conn: DbConn,
  config: Config,
}

#[async_trait]
impl QueueHandler for ScheduledTasksHandler {
  fn new(db_conn: DbConn, config: Config) -> Self {
    Self { db_conn, config }
  }

  fn url(&self) -> String {
    self.config.scheduled_tasks_queue.clone()
  }

  async fn handle(
    &self,
    message: &rusoto_sqs::Message,
    _profile: &rocket::figment::Profile,
  ) -> Result<(), QueueHandlerError> {
    let body = message
      .body
      .clone()
      .ok_or(anyhow!("No body found in sqs message"))?;

    let message_body: SqsMessageBody = serde_json::from_str(&body).map_err(anyhow::Error::from)?;

    let review = self
      .db_conn
      .run(move |conn| {
        reviews::table
          .find(&message_body.review_id)
          .get_result::<Review>(conn)
      })
      .await
      .map_err(anyhow::Error::from)?;

    // if review.state == ReviewState::AwaitingReview {
    self
      .db_conn
      .run(move |conn| {
        diesel::update(&review)
          .set(ReviewChangeset::default().state(ReviewState::Refunded))
          .execute(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;
    // }

    Ok(())
  }
}
