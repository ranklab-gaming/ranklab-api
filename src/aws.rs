pub mod media_convert;
use crate::config::Config;
use rusoto_core::credential::{AwsCredentials, CredentialsError, ProvideAwsCredentials};

pub struct ConfigCredentialsProvider(Config);

impl ConfigCredentialsProvider {
  pub fn new(config: Config) -> Self {
    Self(config)
  }
}

#[async_trait]
impl ProvideAwsCredentials for ConfigCredentialsProvider {
  async fn credentials(&self) -> Result<AwsCredentials, CredentialsError> {
    Ok(AwsCredentials::new(
      self.0.aws_access_key_id.clone(),
      self.0.aws_secret_key.clone(),
      None,
      None,
    ))
  }
}
