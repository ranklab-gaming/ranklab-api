use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub api_key: String,
    pub auth0_domain: String,
    pub s3_bucket: String,
    pub aws_access_key_id: String,
    pub aws_secret_key: String
}
