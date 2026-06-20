//! Smoke tests for the two trickiest spine contracts: the `WaffoRequest`
//! derive (field injection) and the crypto sign/verify roundtrip.

use waffo_rs::base::{InjectCtx, MerchantInfoExt, WaffoRequest};
use waffo_rs::crypto;

#[derive(Default, serde::Serialize, waffo_rs::WaffoRequest)]
struct Nested {
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    merchant_id: Option<String>,
}

impl MerchantInfoExt for Nested {
    fn set_merchant_id_if_empty(&mut self, id: &str) {
        if self.merchant_id.as_deref().unwrap_or("").is_empty() {
            self.merchant_id = Some(id.to_string());
        }
    }
}

#[derive(Default, serde::Serialize, waffo_rs::WaffoRequest)]
struct Req {
    #[waffo(merchant_id)]
    #[serde(skip_serializing_if = "Option::is_none")]
    merchant_id: Option<String>,
    #[waffo(requested_at)]
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_at: Option<String>,
    #[waffo(merchant_info)]
    #[serde(skip_serializing_if = "Option::is_none")]
    merchant_info: Option<Nested>,
}

#[test]
fn derive_injects_all_kinds() {
    let mut r = Req::default();
    let ctx = InjectCtx {
        merchant_id: Some("M123"),
        now: "2026-06-20T00:00:00.000Z",
    };
    r.inject(&ctx);
    assert_eq!(r.merchant_id.as_deref(), Some("M123"));
    assert_eq!(r.requested_at.as_deref(), Some("2026-06-20T00:00:00.000Z"));
    assert_eq!(
        r.merchant_info.unwrap().merchant_id.as_deref(),
        Some("M123")
    );
}

#[test]
fn derive_does_not_overwrite_existing() {
    let mut r = Req {
        merchant_id: Some("explicit".into()),
        requested_at: Some("explicit-ts".into()),
        merchant_info: None,
    };
    let ctx = InjectCtx {
        merchant_id: Some("M123"),
        now: "2026-06-20T00:00:00.000Z",
    };
    r.inject(&ctx);
    assert_eq!(r.merchant_id.as_deref(), Some("explicit"));
    assert_eq!(r.requested_at.as_deref(), Some("explicit-ts"));
}

#[test]
fn crypto_sign_verify_roundtrip() {
    let kp = crypto::generate_key_pair().unwrap();
    let priv_key = crypto::parse_private_key(&kp.private_key).unwrap();
    let pub_key = crypto::parse_public_key(&kp.public_key).unwrap();

    let body = br#"{"hello":"world"}"#;
    let sig = crypto::sign(&priv_key, body).unwrap();
    crypto::verify(&pub_key, body, &sig).unwrap();

    assert!(crypto::verify(&pub_key, b"tampered", &sig).is_err());
}
