//! Cross-cutting concerns: the [`error`] model and the [`trace`] helpers.
//!
//! The SDK only *emits* `tracing` events (at debug/info/warn/error levels);
//! installing a subscriber/provider is the host application's responsibility.

pub mod error;
pub mod trace;

pub use error::{Result, WaffoError};
pub use trace::redact;
