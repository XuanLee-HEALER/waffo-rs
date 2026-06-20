//! Webhook handling: an intentionally thin core (pure functions + one enum,
//! no handler registry).
//!
//! The three pieces:
//! - [`verify_and_parse`] — verify the inbound signature against the raw body
//!   bytes, then parse the `{eventType, result}` envelope into a
//!   [`WebhookEvent`] the caller matches on.
//! - [`WebhookEvent`] — the parsed event variants.
//! - [`build_signed_response`] — produce the exact `{"message":"success"|"failed"}`
//!   response body and its signature (signed with the merchant private key).
//!
//! # The raw-body invariant
//!
//! The webhook signature is computed over the **raw request body bytes**. You
//! MUST verify (and parse) the original bytes — never a re-serialized form. Do
//! not use a body extractor that re-encodes JSON (e.g. axum `Json<T>`); take the
//! raw `Bytes` first, pass them to [`verify_and_parse`], and only then act on
//! the typed event. See [`axum`] for a thin, correct integration.

pub mod events;
pub mod notification;

#[cfg(feature = "axum")]
pub mod axum;

use serde::Deserialize;

use crate::base::Client;
use crate::error::{Result, WaffoError};

pub use events::{
    OrderStatus, RefundStatus, SubscriptionDispatch, SubscriptionStatus, WebhookEvent,
    EVENT_PAYMENT, EVENT_REFUND, EVENT_SUBSCRIPTION_CHANGE, EVENT_SUBSCRIPTION_PERIOD_CHANGED,
    EVENT_SUBSCRIPTION_STATUS, ORDER_STATUS_AUTHED_WAITING_CAPTURE,
    ORDER_STATUS_AUTHORIZATION_REQUIRED, ORDER_STATUS_ORDER_CLOSE, ORDER_STATUS_PAY_IN_PROGRESS,
    ORDER_STATUS_PAY_SUCCESS, REFUND_STATUS_FAILED, REFUND_STATUS_FULLY_REFUNDED,
    REFUND_STATUS_IN_PROGRESS, REFUND_STATUS_PARTIALLY_REFUNDED, SUBSCRIPTION_STATUS_ACTIVE,
    SUBSCRIPTION_STATUS_AUTHORIZATION_REQUIRED, SUBSCRIPTION_STATUS_CHANNEL_CANCELLED,
    SUBSCRIPTION_STATUS_CLOSE, SUBSCRIPTION_STATUS_EXPIRED, SUBSCRIPTION_STATUS_IN_PROGRESS,
    SUBSCRIPTION_STATUS_MERCHANT_CANCELLED, SUBSCRIPTION_STATUS_USER_CANCELLED,
};
pub use notification::{
    FailureReason, PaymentNotificationResult, RefundNotificationResult,
    SubscriptionChangeNotificationResult, SubscriptionNotificationResult,
};

/// Exact success response body Waffo requires (`{"message":"success"}`).
pub const RESPONSE_BODY_SUCCESS: &str = r#"{"message":"success"}"#;
/// Exact failure response body Waffo requires (`{"message":"failed"}`).
pub const RESPONSE_BODY_FAILED: &str = r#"{"message":"failed"}"#;

/// The `{eventType, result}` webhook envelope. `result` is kept as a raw JSON
/// value so it can be deserialized into the variant-specific result struct only
/// after `eventType` routing.
#[derive(Deserialize)]
struct Envelope<'a> {
    #[serde(rename = "eventType", default)]
    event_type: String,
    #[serde(rename = "result", borrow, default)]
    result: Option<&'a serde_json::value::RawValue>,
}

/// Verify the webhook signature over the raw `body` bytes and parse the event.
///
/// Returns [`WaffoError::VerificationFailed`] when `signature` is empty or the
/// signature does not verify against the configured Waffo public key (mirrors
/// Go's "missing signature" / "invalid signature" failures, both of which the
/// caller answers with a signed failure response).
///
/// On success the `{eventType, result}` envelope is routed to the matching
/// [`WebhookEvent`] variant. An unknown `eventType` yields
/// [`WaffoError::Api`] with code `UNKNOWN_EVENT_TYPE`.
pub fn verify_and_parse(client: &Client, body: &[u8], signature: &str) -> Result<WebhookEvent> {
    // A missing/`null` `result` maps to a default-filled result struct rather
    // than an error (Go leaves the `Result` pointer nil without failing).
    fn parse_result<T: serde::de::DeserializeOwned + Default>(raw: &str) -> Result<T> {
        if raw.trim() == "null" {
            Ok(T::default())
        } else {
            Ok(serde_json::from_str(raw)?)
        }
    }

    // 1. Signature: empty or invalid -> verification failed.
    if signature.is_empty() {
        return Err(WaffoError::VerificationFailed);
    }
    crate::crypto::verify(client.public_key(), body, signature)?;

    // 2. Parse the envelope and route on eventType.
    let envelope: Envelope = serde_json::from_slice(body)?;
    let raw = envelope
        .result
        .map_or("null", serde_json::value::RawValue::get);

    let event = match envelope.event_type.as_str() {
        EVENT_PAYMENT => WebhookEvent::Payment(parse_result(raw)?),
        EVENT_REFUND => WebhookEvent::Refund(parse_result(raw)?),
        EVENT_SUBSCRIPTION_STATUS => WebhookEvent::SubscriptionStatus(parse_result(raw)?),
        EVENT_SUBSCRIPTION_PERIOD_CHANGED => {
            WebhookEvent::SubscriptionPeriodChanged(parse_result(raw)?)
        }
        EVENT_SUBSCRIPTION_CHANGE => WebhookEvent::SubscriptionChange(parse_result(raw)?),
        other => {
            return Err(WaffoError::Api {
                code: "UNKNOWN_EVENT_TYPE".to_string(),
                message: format!("unknown webhook event type: {other}"),
            });
        }
    };
    Ok(event)
}

/// Build the signed response body Waffo expects.
///
/// Returns `(body, signature_base64)` where `body` is exactly
/// `{"message":"success"}` when `ok` else `{"message":"failed"}`, and the
/// signature is computed over those exact bytes with the merchant private key.
/// Waffo validates BOTH the HTTP status and this body — the body string must be
/// byte-for-byte one of these two forms.
pub fn build_signed_response(client: &Client, ok: bool) -> Result<(String, String)> {
    let body = if ok {
        RESPONSE_BODY_SUCCESS
    } else {
        RESPONSE_BODY_FAILED
    };
    let signature = crate::crypto::sign(client.private_key(), body.as_bytes())?;
    Ok((body.to_string(), signature))
}
