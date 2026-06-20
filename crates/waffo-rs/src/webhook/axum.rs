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
//! use waffo_rs::webhook::{self, WebhookEvent};
//! use waffo_rs::webhook::axum::{signature_from_headers, signed_response, SIGNATURE_HEADER};
//!
//! async fn webhook_handler(headers: HeaderMap, body: Bytes) -> Response {
//!     let sig = signature_from_headers(&headers).unwrap_or("");
//!     match webhook::verify_and_parse(&CLIENT, &body, sig) {
//!         Ok(WebhookEvent::Payment(p)) => {
//!             // ... your business logic ...
//!             signed_response(&CLIENT, true)
//!         }
//!         Ok(_other) => signed_response(&CLIENT, true),
//!         Err(_) => signed_response(&CLIENT, false),
//!     }
//! }
//! ```

use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;

use crate::base::Client;
use crate::error::Result;

use super::{build_signed_response, verify_and_parse, WebhookEvent};

/// Conventional inbound/outbound signature header name (per the Waffo README
/// examples). Waffo sends the request signature in this header and expects your
/// signed response signature echoed back in it.
pub const SIGNATURE_HEADER: &str = "X-Waffo-Signature";

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
/// HTTP `200 OK` with body `{"message":"success"}` when `ok`, otherwise HTTP
/// `400 Bad Request` with body `{"message":"failed"}`. The response signature is
/// written to the [`SIGNATURE_HEADER`] response header and `Content-Type` is set
/// to `application/json`.
///
/// On the (effectively impossible) signing error this still returns a `400`
/// failed response with no signature header rather than panicking, so a webhook
/// handler always produces a response.
pub fn signed_response(client: &Client, ok: bool) -> Response {
    match build_signed_response(client, ok) {
        Ok((body, signature)) => {
            let status = if ok {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            };
            let mut builder = Response::builder()
                .status(status)
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
fn fallback_failed() -> Response {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(super::RESPONSE_BODY_FAILED))
        .expect("static failed response is always valid")
}
