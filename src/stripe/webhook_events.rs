// use chrono::Utc;
// #[cfg(feature = "webhook-events")]
// use hmac::{Hmac, Mac};
// use serde_derive::{Deserialize, Serialize};
// #[cfg(feature = "webhook-events")]
// use sha2::Sha256;

// use crate::error::WebhookError;
// use crate::ids::EventId;
// use crate::resources::*;

// #[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
// pub enum EventType {
//   #[serde(rename = "order.submitted")]
//   OrderSubmitted,
//   Other(stripe::EventType),
// }

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct WebhookEvent {
//   pub id: EventId,
//   #[serde(rename = "type")]
//   pub event_type: EventType,
//   pub data: EventData,
//   pub livemode: bool,
// }

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct EventData {
//   pub object: EventObject,
// }

// #[derive(Clone, Debug, Deserialize, Serialize)]
// #[serde(tag = "object", rename_all = "snake_case")]
// pub enum EventObject {
//   Order(super::Order),
//   Other(stripe::EventObject),
// }

// #[cfg(feature = "webhook-events")]
// pub struct Webhook;

// #[cfg(feature = "webhook-events")]
// impl Webhook {
//   pub fn construct_event(
//     payload: &str,
//     sig: &str,
//     secret: &str,
//   ) -> Result<WebhookEvent, WebhookError> {
//     // Get Stripe signature from header
//     let signature = Signature::parse(sig)?;
//     let signed_payload = format!("{}.{}", signature.t, payload);

//     // Compute HMAC with the SHA256 hash function, using endpoing secret as key
//     // and signed_payload string as the message.
//     let mut mac =
//       Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| WebhookError::BadKey)?;
//     mac.update(signed_payload.as_bytes());

//     let sig = hex::decode(signature.v1).map_err(|_| WebhookError::BadSignature)?;
//     mac
//       .verify_slice(sig.as_slice())
//       .map_err(|_| WebhookError::BadSignature)?;

//     Ok(serde_json::from_str(payload)?)
//   }
// }

// #[cfg(feature = "webhook-events")]
// #[derive(Debug)]
// struct Signature<'r> {
//   t: i64,
//   v1: &'r str,
//   v0: Option<&'r str>,
// }

// #[cfg(feature = "webhook-events")]
// impl<'r> Signature<'r> {
//   fn parse(raw: &'r str) -> Result<Signature<'r>, WebhookError> {
//     use std::collections::HashMap;
//     let headers: HashMap<&str, &str> = raw
//       .split(',')
//       .map(|header| {
//         let mut key_and_value = header.split('=');
//         let key = key_and_value.next();
//         let value = key_and_value.next();
//         (key, value)
//       })
//       .filter_map(|(key, value)| match (key, value) {
//         (Some(key), Some(value)) => Some((key, value)),
//         _ => None,
//       })
//       .collect();
//     let t = headers.get("t").ok_or(WebhookError::BadSignature)?;
//     let v1 = headers.get("v1").ok_or(WebhookError::BadSignature)?;
//     let v0 = headers.get("v0").map(|r| *r);
//     Ok(Signature {
//       t: t.parse::<i64>().map_err(WebhookError::BadHeader)?,
//       v1,
//       v0,
//     })
//   }
// }
