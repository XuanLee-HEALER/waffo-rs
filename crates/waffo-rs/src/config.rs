//! Configuration: [`WaffoConfig`], its [`ConfigBuilder`], [`Environment`] and
//! per-request [`RequestOptions`].

use crate::common::error::{Result, WaffoError};

/// Target environment; selects the API base URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Environment {
    #[default]
    Sandbox,
    Production,
}

impl Environment {
    /// Base URL (without trailing slash) for this environment.
    pub fn base_url(self) -> &'static str {
        match self {
            Environment::Production => "https://api.waffo.com/api/v1",
            Environment::Sandbox => "https://api-sandbox.waffo.com/api/v1",
        }
    }
}

/// SDK configuration. Build via [`WaffoConfig::builder`] or [`WaffoConfig::from_env`].
#[derive(Debug, Clone)]
pub struct WaffoConfig {
    /// Merchant API key (`X-API-KEY`).
    pub api_key: String,
    /// Merchant private key, Base64 (PKCS#8, PKCS#1 fallback).
    pub private_key: String,
    /// Waffo public key, Base64 (X.509/SPKI, PKCS#1 fallback).
    pub waffo_public_key: String,
    /// Target environment.
    pub environment: Environment,
    /// Optional merchant id; auto-injected into requests when set.
    pub merchant_id: Option<String>,
    /// TCP connect timeout (ms).
    pub connect_timeout_ms: u64,
    /// Read timeout (ms).
    pub read_timeout_ms: u64,
    /// When true, trace logging prints real secrets/bodies (debug only).
    pub debug_unredacted: bool,
}

impl WaffoConfig {
    /// Start building a config.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Base URL for the configured environment.
    pub fn base_url(&self) -> &'static str {
        self.environment.base_url()
    }

    /// Build from `WAFFO_*` environment variables.
    pub fn from_env() -> Result<Self> {
        fn req(key: &str) -> Result<String> {
            std::env::var(key).map_err(|_| WaffoError::Config(format!("missing env var {key}")))
        }
        let environment = match std::env::var("WAFFO_ENVIRONMENT").ok().as_deref() {
            Some("PRODUCTION") => Environment::Production,
            _ => Environment::Sandbox,
        };
        Ok(WaffoConfig {
            api_key: req("WAFFO_API_KEY")?,
            private_key: req("WAFFO_PRIVATE_KEY")?,
            waffo_public_key: req("WAFFO_PUBLIC_KEY")?,
            environment,
            merchant_id: std::env::var("WAFFO_MERCHANT_ID")
                .ok()
                .filter(|s| !s.is_empty()),
            connect_timeout_ms: 10_000,
            read_timeout_ms: 30_000,
            debug_unredacted: false,
        })
    }
}

/// Builder for [`WaffoConfig`].
#[derive(Debug, Clone, Default)]
pub struct ConfigBuilder {
    api_key: Option<String>,
    private_key: Option<String>,
    waffo_public_key: Option<String>,
    environment: Environment,
    merchant_id: Option<String>,
    connect_timeout_ms: Option<u64>,
    read_timeout_ms: Option<u64>,
    debug_unredacted: bool,
}

impl ConfigBuilder {
    #[must_use]
    pub fn api_key(mut self, v: impl Into<String>) -> Self {
        self.api_key = Some(v.into());
        self
    }
    #[must_use]
    pub fn private_key(mut self, v: impl Into<String>) -> Self {
        self.private_key = Some(v.into());
        self
    }
    #[must_use]
    pub fn waffo_public_key(mut self, v: impl Into<String>) -> Self {
        self.waffo_public_key = Some(v.into());
        self
    }
    #[must_use]
    pub fn environment(mut self, v: Environment) -> Self {
        self.environment = v;
        self
    }
    #[must_use]
    pub fn merchant_id(mut self, v: impl Into<String>) -> Self {
        self.merchant_id = Some(v.into());
        self
    }
    #[must_use]
    pub fn connect_timeout_ms(mut self, v: u64) -> Self {
        self.connect_timeout_ms = Some(v);
        self
    }
    #[must_use]
    pub fn read_timeout_ms(mut self, v: u64) -> Self {
        self.read_timeout_ms = Some(v);
        self
    }
    #[must_use]
    pub fn debug_unredacted(mut self, v: bool) -> Self {
        self.debug_unredacted = v;
        self
    }

    /// Validate required fields and produce a [`WaffoConfig`]. Key *format*
    /// validation happens when the [`crate::Client`] is constructed.
    pub fn build(self) -> Result<WaffoConfig> {
        Ok(WaffoConfig {
            api_key: self
                .api_key
                .filter(|s| !s.is_empty())
                .ok_or_else(|| WaffoError::Config("api_key is required".into()))?,
            private_key: self
                .private_key
                .filter(|s| !s.is_empty())
                .ok_or_else(|| WaffoError::Config("private_key is required".into()))?,
            waffo_public_key: self
                .waffo_public_key
                .filter(|s| !s.is_empty())
                .ok_or_else(|| WaffoError::Config("waffo_public_key is required".into()))?,
            environment: self.environment,
            merchant_id: self.merchant_id.filter(|s| !s.is_empty()),
            connect_timeout_ms: self.connect_timeout_ms.unwrap_or(10_000),
            read_timeout_ms: self.read_timeout_ms.unwrap_or(30_000),
            debug_unredacted: self.debug_unredacted,
        })
    }
}

/// Per-request overrides (timeouts and extra headers).
#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    pub connect_timeout_ms: Option<u64>,
    pub read_timeout_ms: Option<u64>,
    pub headers: Vec<(String, String)>,
}
