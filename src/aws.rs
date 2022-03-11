use rusoto_core::credential::{AwsCredentials, CredentialsError, ProvideAwsCredentials};

pub struct CredentialsProvider {
  access_key_id: String,
  secret_key: String,
}

impl CredentialsProvider {
  pub fn new(access_key_id: String, secret_key: String) -> Self {
    Self {
      access_key_id,
      secret_key,
    }
  }
}

#[async_trait]
impl ProvideAwsCredentials for CredentialsProvider {
  async fn credentials(&self) -> Result<AwsCredentials, CredentialsError> {
    Ok(AwsCredentials::new(
      self.access_key_id.clone(),
      self.secret_key.clone(),
      None,
      None,
    ))
  }
}
