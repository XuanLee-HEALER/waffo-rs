//! Transport core: the [`Client`], the [`Endpoint`] abstraction and the single
//! uniform request/response processing path ([`send`] / [`send_with`]).
//!
//! Every endpoint is a pure data declaration (`impl Endpoint`); all processing
//! — inject, serialize, sign, send, verify, envelope, error-map — lives here
//! and is applied identically to all endpoints. The pipeline is fixed: the SDK
//! does not support injecting business logic / middleware.

use std::time::Duration;

use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::common::error::{Result, WaffoError};
use crate::common::trace::redact;
use crate::config::{RequestOptions, WaffoConfig};
use crate::crypto;

pub const HEADER_API_KEY: &str = "X-API-KEY";
pub const HEADER_SIGNATURE: &str = "X-SIGNATURE";
pub const HEADER_API_VERSION: &str = "X-API-VERSION";
pub const HEADER_SDK_VERSION: &str = "X-SDK-VERSION";
pub const API_VERSION: &str = "1.0.0";
pub const SDK_VERSION: &str = concat!("waffo-rs/", env!("CARGO_PKG_VERSION"));

/// Dynamic-params escape hatch (Go `extraParams`); transparently passed through.
pub type ExtraParams = serde_json::Map<String, serde_json::Value>;

/// Context handed to [`WaffoRequest::inject`] before a request is serialized.
pub struct InjectCtx<'a> {
    pub merchant_id: Option<&'a str>,
    pub now: &'a str,
}

/// Implemented by every request params struct (via `#[derive(WaffoRequest)]`).
/// Performs the pre-send field injection the Go SDK did via reflection.
pub trait WaffoRequest {
    /// Inject the configured merchant id / current timestamp into this request
    /// before it is serialized and signed.
    fn inject(&mut self, ctx: &InjectCtx<'_>);
}

/// Implemented by nested merchant-info structs so the derive can inject the
/// configured merchant id into them.
pub trait MerchantInfoExt {
    /// Set the merchant id on this nested struct if it is currently empty.
    fn set_merchant_id_if_empty(&mut self, merchant_id: &str);
}

/// A single API endpoint: a pure declaration of request/response types + path.
pub trait Endpoint {
    /// Request params type — serialized and signed as the request body.
    type Req: WaffoRequest + Serialize;
    /// Response data type — decoded from the envelope `data` field.
    type Resp: DeserializeOwned;
    /// Path appended to the environment base URL, e.g. `"/order/create"`.
    const PATH: &'static str;
    /// Read/idempotent endpoints map transport failures to `UnknownStatus`
    /// (Go's "E0001 on inquiry/capture/..." semantics).
    const READ: bool;
}

/// The unified `{code, msg, data}` response envelope.
#[derive(serde::Deserialize)]
pub struct Envelope {
    pub code: String,
    #[serde(default)]
    pub msg: String,
    #[serde(default)]
    pub data: Option<Box<serde_json::value::RawValue>>,
}

impl Envelope {
    /// Map the envelope to `Ok(data)` / `Err(business error)` by `code`.
    pub fn into_result<T: DeserializeOwned>(self) -> Result<T> {
        match self.code.as_str() {
            "0" => {
                let raw = self
                    .data
                    .as_deref()
                    .map_or("null", serde_json::value::RawValue::get);
                Ok(serde_json::from_str(raw)?)
            }
            "E0001" => Err(WaffoError::UnknownStatus {
                code: self.code,
                message: self.msg,
            }),
            _ => Err(WaffoError::Api {
                code: self.code,
                message: self.msg,
            }),
        }
    }
}

/// Holds config, the HTTP client and the parsed keys (keys parsed once).
pub struct Client {
    config: WaffoConfig,
    http: reqwest::Client,
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl Client {
    /// Construct a client; parses the keys (surfacing format errors) and builds
    /// a default HTTP client honoring the configured timeouts.
    pub fn new(config: WaffoConfig) -> Result<Self> {
        let private_key = crypto::parse_private_key(&config.private_key)?;
        let public_key = crypto::parse_public_key(&config.waffo_public_key)?;
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_millis(config.connect_timeout_ms))
            .timeout(Duration::from_millis(config.read_timeout_ms))
            .build()
            .map_err(WaffoError::Transport)?;
        let client = Client {
            config,
            http,
            private_key,
            public_key,
        };
        tracing::debug!(
            environment = ?client.config.environment,
            base_url = client.config.base_url(),
            "waffo client created",
        );
        Ok(client)
    }

    /// Construct a client with a caller-provided `reqwest::Client` (proxy, TLS,
    /// custom pool, test transport). Keys are still parsed here.
    pub fn with_http_client(config: WaffoConfig, http: reqwest::Client) -> Result<Self> {
        let private_key = crypto::parse_private_key(&config.private_key)?;
        let public_key = crypto::parse_public_key(&config.waffo_public_key)?;
        let client = Client {
            config,
            http,
            private_key,
            public_key,
        };
        tracing::debug!(
            environment = ?client.config.environment,
            base_url = client.config.base_url(),
            "waffo client created (custom http client)",
        );
        Ok(client)
    }

    /// The configuration this client was built with.
    pub fn config(&self) -> &WaffoConfig {
        &self.config
    }

    /// Parsed merchant private key (used by the webhook responder to sign).
    pub fn private_key(&self) -> &RsaPrivateKey {
        &self.private_key
    }

    /// Parsed Waffo public key (used to verify responses / webhooks).
    pub fn public_key(&self) -> &RsaPublicKey {
        &self.public_key
    }

    pub(crate) fn sign(&self, body: &[u8]) -> Result<String> {
        crypto::sign(&self.private_key, body)
    }

    async fn post(
        &self,
        path: &str,
        body: Vec<u8>,
        signature: &str,
        opts: Option<&RequestOptions>,
    ) -> Result<(reqwest::header::HeaderMap, Vec<u8>)> {
        let url = format!("{}{}", self.config.base_url(), path);
        let mut req = self
            .http
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(HEADER_API_KEY, &self.config.api_key)
            .header(HEADER_SIGNATURE, signature)
            .header(HEADER_API_VERSION, API_VERSION)
            .header(HEADER_SDK_VERSION, SDK_VERSION)
            .body(body);
        if let Some(o) = opts {
            if let Some(t) = o.read_timeout_ms {
                req = req.timeout(Duration::from_millis(t));
            }
            for (k, v) in &o.headers {
                req = req.header(k.as_str(), v.as_str());
            }
        }
        let resp = req.send().await?;
        let headers = resp.headers().clone();
        let bytes = resp.bytes().await?.to_vec();
        Ok((headers, bytes))
    }

    fn verify_response(&self, headers: &reqwest::header::HeaderMap, body: &[u8]) -> Result<()> {
        if let Some(sig) = headers.get(HEADER_SIGNATURE).and_then(|v| v.to_str().ok()) {
            crypto::verify(&self.public_key, body, sig)?;
        }
        Ok(())
    }
}

/// Run an endpoint through the uniform processing path with default options.
pub async fn send<E: Endpoint>(client: &Client, req: E::Req) -> Result<E::Resp> {
    send_with::<E>(client, req, None).await
}

/// Run an endpoint through the uniform processing path with per-request options.
pub async fn send_with<E: Endpoint>(
    client: &Client,
    mut req: E::Req,
    opts: Option<&RequestOptions>,
) -> Result<E::Resp> {
    let unredacted = client.config.debug_unredacted;

    // ---- request side ----
    let now = crypto::now_iso8601();
    let ctx = InjectCtx {
        merchant_id: client.config.merchant_id.as_deref(),
        now: &now,
    };
    req.inject(&ctx);
    let body = serde_json::to_vec(&req)?;
    tracing::info!(method = "POST", path = E::PATH, "waffo request");

    let signature = match client.sign(&body) {
        Ok(sig) => sig,
        Err(e) => {
            tracing::error!(path = E::PATH, error = %e, "request signing failed");
            return Err(e);
        }
    };
    tracing::debug!(
        path = E::PATH,
        body = %redact(&String::from_utf8_lossy(&body), unredacted),
        signature = %redact(&signature, unredacted),
        "signed request",
    );

    // ---- send (read endpoints fold transport failures into UnknownStatus) ----
    let (headers, bytes) = match client.post(E::PATH, body, &signature, opts).await {
        Ok(v) => v,
        Err(e) => {
            if E::READ {
                tracing::warn!(
                    path = E::PATH,
                    error = %e,
                    "transport failure on read endpoint; reporting unknown status",
                );
                return Err(WaffoError::UnknownStatus {
                    code: "E0001".to_string(),
                    message: e.to_string(),
                });
            }
            tracing::error!(path = E::PATH, error = %e, "transport error");
            return Err(e);
        }
    };

    // ---- response side ----
    if let Err(e) = client.verify_response(&headers, &bytes) {
        tracing::error!(path = E::PATH, "response signature verification failed");
        return Err(e);
    }
    tracing::debug!(
        path = E::PATH,
        response = %redact(&String::from_utf8_lossy(&bytes), unredacted),
        "received response",
    );

    let envelope: Envelope = serde_json::from_slice(&bytes)?;
    match envelope.into_result::<E::Resp>() {
        Ok(resp) => {
            tracing::info!(path = E::PATH, "waffo request succeeded");
            Ok(resp)
        }
        Err(e) => {
            match &e {
                WaffoError::UnknownStatus { code, .. } => {
                    tracing::warn!(path = E::PATH, code = code.as_str(), "waffo unknown status");
                }
                WaffoError::Api { code, .. } => {
                    tracing::warn!(path = E::PATH, code = code.as_str(), "waffo business error");
                }
                other => {
                    tracing::error!(path = E::PATH, error = %other, "response decode failed");
                }
            }
            Err(e)
        }
    }
}
