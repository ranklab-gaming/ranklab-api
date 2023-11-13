use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
  pub auth_client_secret: String,
  pub avatar_processor_lambda_arn: String,
  pub aws_access_key_id: String,
  pub aws_secret_key: String,
  pub host: String,
  pub instance_id: Option<String>,
  pub intercom_access_token: Option<String>,
  pub intercom_verification_secret: Option<String>,
  pub media_convert_queue_arn: String,
  pub media_convert_role_arn: String,
  pub rekognition_queue_url: String,
  pub rekognition_role_arn: String,
  pub rekognition_topic_arn: String,
  pub sentry_dsn: Option<String>,
  pub uploads_bucket: String,
  pub uploads_queue_url: String,
  pub web_host: String,
}
