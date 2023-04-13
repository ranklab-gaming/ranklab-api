use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct OidcConfiguration {
  pub jwks_uri: String,
  pub issuer: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum KeyType {
  #[serde(rename = "RSA")]
  Rsa,
}

#[derive(Debug, Clone, Deserialize)]
pub enum KeyAlgorithm {
  RS256,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Jwk {
  #[serde(rename = "kty")]
  _kty: KeyType,
  #[serde(rename = "alg")]
  _alg: KeyAlgorithm,
  pub kid: String,
  pub n: String,
  pub e: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Jwks {
  pub keys: Vec<Jwk>,
}

pub struct OidcCache {
  pub oidc_configuration: OidcConfiguration,
  pub jwks: Jwks,
}

async fn fetch_oidc_configuration(web_host: &str) -> Result<OidcConfiguration, reqwest::Error> {
  let oidc_configuration_url = format!("{}{}", web_host, "/oidc/.well-known/openid-configuration");
  reqwest::get(&oidc_configuration_url)
    .await?
    .json::<OidcConfiguration>()
    .await
}

async fn fetch_jwks(jwks_uri: &str) -> Result<Jwks, reqwest::Error> {
  reqwest::get(jwks_uri).await?.json::<Jwks>().await
}

pub async fn init_cache(web_host: &str) -> Result<OidcCache, reqwest::Error> {
  let oidc_configuration = fetch_oidc_configuration(web_host).await?;
  let jwks = fetch_jwks(&oidc_configuration.jwks_uri).await?;

  Ok(OidcCache {
    oidc_configuration,
    jwks,
  })
}
