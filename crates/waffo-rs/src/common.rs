//! Cross-cutting helpers. The SDK only emits `tracing` events; installing a
//! subscriber/provider is the host application's responsibility.

/// Redact a secret for logging unless unredacted debug logging is enabled.
///
/// Used so request/response bodies, signatures, and keys never reach trace
/// output by default. Pass `debug_unredacted = true` (config flag) only for
/// local debugging — never in production.
pub fn redact(value: &str, debug_unredacted: bool) -> String {
    if debug_unredacted {
        value.to_string()
    } else if value.is_empty() {
        String::new()
    } else {
        "***redacted***".to_string()
    }
}
