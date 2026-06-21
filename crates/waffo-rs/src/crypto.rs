//! RSA signing / verification, key parsing, key-pair generation and the
//! ISO-8601 timestamp helper. Signature scheme: RSA PKCS#1 v1.5 over SHA-256,
//! Base64-encoded — matching the Go SDK.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey};
use rsa::{Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};

use crate::common::error::{Result, WaffoError};

/// Timestamp format used for all `*RequestedAt` fields (UTC, 3-digit millis).
pub const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";

/// Current UTC time as an ISO-8601 string (`2026-06-20T12:34:56.789Z`).
pub fn now_iso8601() -> String {
    chrono::Utc::now().format(TIMESTAMP_FORMAT).to_string()
}

/// Parse a Base64-encoded private key (PKCS#8, falling back to PKCS#1).
pub fn parse_private_key(b64: &str) -> Result<RsaPrivateKey> {
    let der = STANDARD
        .decode(b64.trim())
        .map_err(|e| WaffoError::InvalidPrivateKey(e.to_string()))?;
    if let Ok(key) = RsaPrivateKey::from_pkcs8_der(&der) {
        return Ok(key);
    }
    RsaPrivateKey::from_pkcs1_der(&der).map_err(|e| WaffoError::InvalidPrivateKey(e.to_string()))
}

/// Parse a Base64-encoded public key (X.509/SPKI, falling back to PKCS#1).
pub fn parse_public_key(b64: &str) -> Result<RsaPublicKey> {
    let der = STANDARD
        .decode(b64.trim())
        .map_err(|e| WaffoError::InvalidPublicKey(e.to_string()))?;
    if let Ok(key) = RsaPublicKey::from_public_key_der(&der) {
        return Ok(key);
    }
    RsaPublicKey::from_pkcs1_der(&der).map_err(|e| WaffoError::InvalidPublicKey(e.to_string()))
}

/// Sign `body` with `key`; returns a Base64 signature.
pub fn sign(key: &RsaPrivateKey, body: &[u8]) -> Result<String> {
    let hashed = Sha256::digest(body);
    let signature = key
        .sign(Pkcs1v15Sign::new::<Sha256>(), &hashed)
        .map_err(|e| WaffoError::SigningFailed(e.to_string()))?;
    Ok(STANDARD.encode(signature))
}

/// Verify a Base64 `signature` over `body` against `key`.
pub fn verify(key: &RsaPublicKey, body: &[u8], signature: &str) -> Result<()> {
    let sig = STANDARD
        .decode(signature.trim())
        .map_err(|_| WaffoError::VerificationFailed)?;
    let hashed = Sha256::digest(body);
    key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed, &sig)
        .map_err(|_| WaffoError::VerificationFailed)
}

/// A generated RSA-2048 key pair (Base64; private PKCS#8, public X.509/SPKI).
#[derive(Debug, Clone)]
pub struct KeyPair {
    pub private_key: String,
    pub public_key: String,
}

/// Generate an RSA-2048 key pair for testing / key rotation.
pub fn generate_key_pair() -> Result<KeyPair> {
    let mut rng = rand::thread_rng();
    let private =
        RsaPrivateKey::new(&mut rng, 2048).map_err(|e| WaffoError::SigningFailed(e.to_string()))?;
    let public = private.to_public_key();
    let private_der = private
        .to_pkcs8_der()
        .map_err(|e| WaffoError::InvalidPrivateKey(e.to_string()))?;
    let public_der = public
        .to_public_key_der()
        .map_err(|e| WaffoError::InvalidPublicKey(e.to_string()))?;
    Ok(KeyPair {
        private_key: STANDARD.encode(private_der.as_bytes()),
        public_key: STANDARD.encode(public_der.as_bytes()),
    })
}
