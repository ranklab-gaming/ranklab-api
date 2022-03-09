use hex;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use stripe::{EventId, WebhookError};

use super::order::Order;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum EventTypeExt {
  #[serde(rename = "order.completed")]
  OrderCompleted,
  #[serde(rename = "order.payment_succeeded")]
  OrderPaymentSucceeded,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum EventType {
  Ext(EventTypeExt),
  Other(stripe::EventType),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebhookEvent {
  pub id: EventId,
  #[serde(rename = "type")]
  pub event_type: EventType,
  pub data: EventData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventData {
  pub object: EventObject,
  pub livemode: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "object", rename_all = "snake_case")]
pub enum EventObjectExt {
  Order(Order),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EventObject {
  Ext(EventObjectExt),
  Other(stripe::EventObject),
}

pub struct Webhook;

impl Webhook {
  pub fn construct_event(
    payload: &str,
    sig: &str,
    secret: &str,
  ) -> Result<WebhookEvent, WebhookError> {
    // Get Stripe signature from header
    let signature = Signature::parse(sig)?;
    let signed_payload = format!("{}.{}", signature.t, payload);

    // Compute HMAC with the SHA256 hash function, using endpoing secret as key
    // and signed_payload string as the message.
    let mut mac =
      Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| WebhookError::BadKey)?;
    mac.update(signed_payload.as_bytes());

    let sig = hex::decode(signature.v1).map_err(|_| WebhookError::BadSignature)?;
    mac
      .verify_slice(sig.as_slice())
      .map_err(|_| WebhookError::BadSignature)?;

    Ok(serde_json::from_str(payload)?)
  }
}

#[derive(Debug)]
struct Signature<'r> {
  t: i64,
  v1: &'r str,
}

impl<'r> Signature<'r> {
  fn parse(raw: &'r str) -> Result<Signature<'r>, WebhookError> {
    use std::collections::HashMap;
    let headers: HashMap<&str, &str> = raw
      .split(',')
      .map(|header| {
        let mut key_and_value = header.split('=');
        let key = key_and_value.next();
        let value = key_and_value.next();
        (key, value)
      })
      .filter_map(|(key, value)| match (key, value) {
        (Some(key), Some(value)) => Some((key, value)),
        _ => None,
      })
      .collect();
    let t = headers.get("t").ok_or(WebhookError::BadSignature)?;
    let v1 = headers.get("v1").ok_or(WebhookError::BadSignature)?;
    let v0 = headers.get("v0").map(|r| *r);
    Ok(Signature {
      t: t.parse::<i64>().map_err(WebhookError::BadHeader)?,
      v1,
    })
  }
}
