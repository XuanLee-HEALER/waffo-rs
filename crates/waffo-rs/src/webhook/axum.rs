//! Thin axum integration (feature `axum`).
//!
//! These helpers are intentionally minimal: the SDK does not own a router or a
//! handler registry. You write your own handler, extract the raw body bytes and
//! the signature header yourself, call [`verify_and_parse`], `match` on the
//! returned [`WebhookEvent`], and turn your decision back into a response with
//! [`signed_response`].
//!
//! # The raw-body invariant (critical)
//!
//! The webhook signature is computed over the **raw request body bytes**. In an
//! axum handler take [`axum::body::Bytes`] (or `String`) as the body extractor —
//! never `Json<T>`, which re-serializes and would invalidate the signature.
//!
//! ```ignore
//! use axum::{body::Bytes, http::HeaderMap, response::Response};
//! use waffo_rs::webhook::{self, WebhookAck, WebhookEvent};
//! use waffo_rs::webhook::axum::{signature_from_headers, signed_response, SIGNATURE_HEADER};
//!
//! async fn webhook_handler(headers: HeaderMap, body: Bytes) -> Response {
//!     let sig = signature_from_headers(&headers).unwrap_or("");
//!     match webhook::verify_and_parse(&CLIENT, &body, sig) {
//!         Ok(WebhookEvent::Payment(p)) => match process_payment(&p) {
//!             Ok(()) => signed_response(&CLIENT, WebhookAck::Success),
//!             // Transient failure: ask Waffo to retry (up to 24h).
//!             Err(_) => signed_response(&CLIENT, WebhookAck::Unknown),
//!         },
//!         Ok(_other) => signed_response(&CLIENT, WebhookAck::Success),
//!         // Bad signature etc. — don't acknowledge; Waffo retries.
//!         Err(_) => signed_response(&CLIENT, WebhookAck::Failed),
//!     }
//! }
//! ```

use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;

use crate::base::Client;
use crate::common::error::Result;

use super::{WebhookAck, WebhookEvent, build_signed_response, verify_and_parse};

/// Inbound/outbound webhook signature header name. Waffo sends the request
/// signature in this header and expects your signed response signature echoed
/// back in it.
///
/// This is `X-SIGNATURE` — the same header the main API channel uses, per the
/// Chargeback/API spec and the official Go SDK's webhook server (which reads and
/// writes `X-SIGNATURE`). Verified end-to-end against the sandbox.
pub const SIGNATURE_HEADER: &str = "X-SIGNATURE";

/// Read the webhook signature from a request [`HeaderMap`].
///
/// Looks up [`SIGNATURE_HEADER`] (case-insensitive, as HTTP header maps are) and
/// returns its UTF-8 value, or `None` when absent / non-UTF-8. Pass the result
/// (or `""`) straight into [`verify_and_parse`]; an empty/missing signature is
/// treated as a verification failure there.
pub fn signature_from_headers(headers: &HeaderMap) -> Option<&str> {
    headers.get(SIGNATURE_HEADER).and_then(|v| v.to_str().ok())
}

/// Verify + parse an inbound webhook from raw axum primitives.
///
/// Thin wrapper over [`verify_and_parse`] that pulls the signature out of
/// `headers` for you. `body` must be the **raw** request bytes (e.g. an
/// [`axum::body::Bytes`] extractor), not a re-serialized form.
pub fn parse_request(client: &Client, headers: &HeaderMap, body: &[u8]) -> Result<WebhookEvent> {
    let signature = signature_from_headers(headers).unwrap_or("");
    verify_and_parse(client, body, signature)
}

/// Turn [`build_signed_response`] into an axum [`Response`].
///
/// Always HTTP `200 OK` (matching the Waffo reference server) with the
/// [`WebhookAck`] body — `{"message":"success"|"failed"|"unknown"}` — and the
/// response signature in the [`SIGNATURE_HEADER`] header; `Content-Type` is
/// `application/json`. Waffo decides whether to retry from the body, not the
/// status: only [`WebhookAck::Success`] acknowledges. Accepts a [`WebhookAck`]
/// or a `bool` (`true` → success, `false` → failed).
///
/// On the (effectively impossible) signing error this returns a `200` failed
/// response with no signature header rather than panicking, so a webhook handler
/// always produces a response (Waffo then retries, since it can't verify it).
pub fn signed_response(client: &Client, ack: impl Into<WebhookAck>) -> Response {
    match build_signed_response(client, ack.into()) {
        Ok((body, signature)) => {
            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header(axum::http::header::CONTENT_TYPE, "application/json");
            if let Ok(value) = HeaderValue::from_str(&signature) {
                builder = builder.header(SIGNATURE_HEADER, value);
            }
            builder
                .body(Body::from(body))
                .unwrap_or_else(|_| fallback_failed())
        }
        Err(_) => fallback_failed(),
    }
}

/// Last-resort failed response (used only if signing or response building fails).
/// Unsigned, so Waffo cannot verify it and will retry — the desired effect when
/// we could not produce a proper reply.
fn fallback_failed() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(super::RESPONSE_BODY_FAILED))
        .expect("static failed response is always valid")
}
