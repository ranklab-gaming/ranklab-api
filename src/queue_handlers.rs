use crate::config::Config;
use crate::guards::DbConn;

mod s3_bucket;
mod stripe;

pub use self::stripe::StripeHandler;
pub use s3_bucket::S3BucketHandler;

#[async_trait]
pub trait QueueHandler: Send + Sync {
  fn new(db_conn: DbConn, config: Config) -> Self;
  fn url(&self) -> String;

  async fn handle(
    &self,
    message: &rusoto_sqs::Message,
    profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()>;
}
