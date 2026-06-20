//! Tracing helpers.
//!
//! The SDK emits `tracing` events at the standard levels:
//!
//! - **error** — SDK-side failures (signing, response/webhook verification,
//!   transport errors on write operations);
//! - **warn**  — server-reported problems (business error codes, unknown
//!   status, transport failures folded into `UnknownStatus`);
//! - **info**  — request lifecycle (one event per request, plus success), with
//!   no sensitive data;
//! - **debug** — verbose detail (request/response bodies and signatures),
//!   redacted by default (see [`redact`]).
//!
//! Installing a subscriber is the host application's responsibility.

/// Redact a secret for logging unless unredacted debug logging is enabled.
///
/// Used so request/response bodies, signatures, and keys never reach trace
/// output by default. Pass `debug_unredacted = true` (the
/// [`crate::WaffoConfig::debug_unredacted`] flag) only for local debugging —
/// never in production.
pub fn redact(value: &str, debug_unredacted: bool) -> String {
    if debug_unredacted {
        value.to_string()
    } else if value.is_empty() {
        String::new()
    } else {
        "***redacted***".to_string()
    }
}
