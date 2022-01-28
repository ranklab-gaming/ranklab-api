use crate::config::Config;
use crate::fairings::sqs::QueueHandler;
use crate::guards::DbConn;
use diesel::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct RecordS3Object {
  key: String,
}

#[derive(Deserialize)]
struct RecordS3 {
  object: RecordS3Object,
}

#[derive(Deserialize)]
struct Record {
  s3: RecordS3,
}

#[derive(Deserialize)]
struct SqsMessageBody {
  #[serde(rename = "Records")]
  records: Vec<Record>,
}

pub struct S3BucketHandler {
  db_conn: DbConn,
  config: Config,
}

#[async_trait]
impl QueueHandler for S3BucketHandler {
  fn new(db_conn: DbConn, config: Config) -> Self {
    Self { db_conn, config }
  }

  fn url(&self) -> String {
    self.config.s3_bucket_queue.clone()
  }

  async fn handle(&self, message: &rusoto_sqs::Message) -> anyhow::Result<()> {
    use crate::schema::recordings::dsl::*;

    let body = message
      .body
      .clone()
      .ok_or(anyhow::anyhow!("No body in message"))?;
    let message_body: SqsMessageBody = serde_json::from_str(&body)?;

    for record in message_body.records {
      self
        .db_conn
        .run::<_, diesel::result::QueryResult<_>>(move |conn| {
          let existing_recording = recordings.filter(video_key.eq(&record.s3.object.key));

          diesel::update(existing_recording)
            .set(uploaded.eq(true))
            .execute(conn)?;

          Ok(())
        })
        .await?;
    }

    Ok(())
  }
}
