use std::error::Error;
use std::fmt::Formatter;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, SystemTimeError};

use async_mutex::Mutex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

/// Auth0 OAuth token.
#[derive(Deserialize)]
pub struct Token {
  // scope: String,
  // token_type: String,
  /// Token expiration in seconds.
  pub expires_in: u64,
  /// Encoded JWT access token.
  pub access_token: String,
}

#[derive(Debug)]
pub enum TokenError {
  Time(SystemTimeError),
  Transport(reqwest::Error),
  AccessDenied(String),
}

#[derive(Serialize, Clone, Debug)]
struct TokenOpts {
  audience: String,
  grant_type: String,
  client_id: String,
  client_secret: String,
}

#[derive(Deserialize)]
struct TokenErrorResponse {
  error_description: String,
}

/// Provides oauth token retrieval and expiration checks.
#[derive(Debug)]
pub struct TokenManager {
  issuer_base_url: String,
  token: Mutex<Option<String>>,
  token_opts: TokenOpts,
  token_expiration: AtomicU64,
}

impl TokenManager {
  /// Gets builder for [TokenManager].
  pub fn new(issuer_base_url: &str, audience: &str, client_id: &str, client_secret: &str) -> Self {
    Self {
      issuer_base_url: issuer_base_url.to_owned(),
      token: Mutex::new(None),
      token_opts: TokenOpts {
        audience: audience.to_owned(),
        grant_type: "client_credentials".to_owned(),
        client_id: client_id.to_owned(),
        client_secret: client_secret.to_owned(),
      },
      token_expiration: AtomicU64::new(0),
    }
  }

  /// Gets valid encoded JWT token.
  pub async fn get_token(&self) -> Result<String, TokenError> {
    let now = SystemTime::now();
    let expiration =
      SystemTime::UNIX_EPOCH + Duration::from_secs(self.token_expiration.load(Ordering::SeqCst));

    if now > expiration {
      return self.fetch_token().await;
    }

    {
      let token = self.token.lock().await;
      if let Some(token) = token.deref() {
        return Ok(token.to_string());
      }
    }

    self.fetch_token().await
  }

  /// Gets new encoded JWT token from auth0.
  async fn fetch_token(&self) -> Result<String, TokenError> {
    let client = reqwest::Client::new();

    let res = client
      .post(&format!("{}oauth/token", self.issuer_base_url))
      .form(&self.token_opts)
      .send()
      .await?;

    if res.status() != StatusCode::OK {
      return Err(res.json::<TokenErrorResponse>().await?.into());
    }

    let token: Token = res.json().await?;

    *self.token.lock().await = Some(token.access_token.clone());
    self.token_expiration.store(
      (SystemTime::now() + Duration::from_secs(token.expires_in))
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs(),
      Ordering::SeqCst,
    );

    Ok(token.access_token)
  }
}

impl std::fmt::Display for TokenError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl From<TokenErrorResponse> for TokenError {
  fn from(res: TokenErrorResponse) -> TokenError {
    TokenError::AccessDenied(res.error_description)
  }
}

impl From<reqwest::Error> for TokenError {
  fn from(err: reqwest::Error) -> TokenError {
    TokenError::Transport(err)
  }
}

impl Error for TokenError {}
