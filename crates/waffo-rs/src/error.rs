//! Error model: a single public [`WaffoError`] enum. Source errors (reqwest /
//! serde / crypto) are absorbed via `From`; business errors are mapped from the
//! server response `code`.

use thiserror::Error;

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, WaffoError>;

/// The single public error type for the SDK.
#[derive(Debug, Error)]
pub enum WaffoError {
    // ---- raw layer (source errors, not surfaced as separate public types) ----
    /// The configured private key could not be parsed.
    #[error("invalid private key: {0}")]
    InvalidPrivateKey(String),

    /// The configured Waffo public key could not be parsed.
    #[error("invalid public key: {0}")]
    InvalidPublicKey(String),

    /// Signing the request body failed.
    #[error("signing failed: {0}")]
    SigningFailed(String),

    /// A response or webhook signature failed verification.
    #[error("signature verification failed")]
    VerificationFailed,

    /// Invalid configuration (missing/invalid builder or env input).
    #[error("configuration error: {0}")]
    Config(String),

    /// JSON (de)serialization failure.
    #[error("serialization error: {0}")]
    Serialize(#[from] serde_json::Error),

    /// Underlying HTTP transport failure.
    #[error("network error: {0}")]
    Transport(#[from] reqwest::Error),

    // ---- business layer (mapped from the server response `code`) ----
    /// Server returned a non-success business code (`code != "0"`).
    #[error("api error [{code}]: {message}")]
    Api { code: String, message: String },

    /// Status is uncertain (`code == "E0001"`, or a read/idempotent call's
    /// transport failed). The caller should re-inquire rather than assume
    /// failure.
    #[error("unknown status [{code}]: {message}")]
    UnknownStatus { code: String, message: String },
}

impl WaffoError {
    /// True for the [`WaffoError::UnknownStatus`] variant.
    pub fn is_unknown_status(&self) -> bool {
        matches!(self, WaffoError::UnknownStatus { .. })
    }

    /// The server business code, when this error carries one.
    pub fn api_code(&self) -> Option<&str> {
        match self {
            WaffoError::Api { code, .. } | WaffoError::UnknownStatus { code, .. } => Some(code),
            _ => None,
        }
    }
}
