//! Unit tests for the error model, the response envelope, the trace redactor
//! and `Client` construction (`waffo_rs::{error, base, common}`).

use waffo_rs::base::Envelope;
use waffo_rs::common::redact;
use waffo_rs::{crypto, Client, WaffoConfig, WaffoError};

// ---- error helpers ---------------------------------------------------------

#[test]
fn error_is_unknown_status_and_api_code() {
    let api = WaffoError::Api {
        code: "A0003".into(),
        message: "bad".into(),
    };
    assert!(!api.is_unknown_status());
    assert_eq!(api.api_code(), Some("A0003"));

    let unknown = WaffoError::UnknownStatus {
        code: "E0001".into(),
        message: "uncertain".into(),
    };
    assert!(unknown.is_unknown_status());
    assert_eq!(unknown.api_code(), Some("E0001"));

    let other = WaffoError::VerificationFailed;
    assert!(!other.is_unknown_status());
    assert_eq!(other.api_code(), None);
}

#[test]
fn error_display_includes_context() {
    let api = WaffoError::Api {
        code: "A0014".into(),
        message: "refund rejected".into(),
    };
    let text = api.to_string();
    assert!(text.contains("A0014"));
    assert!(text.contains("refund rejected"));
    assert!(WaffoError::Config("oops".into()).to_string().contains("oops"));
}

// ---- envelope -> result ----------------------------------------------------

#[derive(Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
struct Data {
    #[serde(default)]
    x: i64,
}

fn envelope(json: &str) -> Envelope {
    serde_json::from_str(json).expect("envelope should deserialize")
}

#[test]
fn envelope_success_decodes_data() {
    let d: Data = envelope(r#"{"code":"0","msg":"ok","data":{"x":7}}"#)
        .into_result()
        .expect("code 0 should decode the data");
    assert_eq!(d, Data { x: 7 });
}

#[test]
fn envelope_e0001_is_unknown_status() {
    let err = envelope(r#"{"code":"E0001","msg":"uncertain"}"#)
        .into_result::<Data>()
        .unwrap_err();
    assert!(err.is_unknown_status());
    assert_eq!(err.api_code(), Some("E0001"));
}

#[test]
fn envelope_nonzero_code_is_api_error() {
    let err = envelope(r#"{"code":"A0014","msg":"refund rejected"}"#)
        .into_result::<Data>()
        .unwrap_err();
    assert!(!err.is_unknown_status());
    assert!(matches!(err, WaffoError::Api { .. }));
    assert_eq!(err.api_code(), Some("A0014"));
}

#[test]
fn envelope_success_without_data_errors() {
    // code 0 but no data -> decodes from "null", which a struct rejects.
    let result = envelope(r#"{"code":"0"}"#).into_result::<Data>();
    assert!(result.is_err());
}

// ---- redaction -------------------------------------------------------------

#[test]
fn redact_hides_unless_debug() {
    assert_eq!(redact("secret", false), "***redacted***");
    assert_eq!(redact("secret", true), "secret");
    assert_eq!(redact("", false), "");
    assert_eq!(redact("", true), "");
}

// ---- client construction ---------------------------------------------------

fn key_pair() -> (String, String) {
    let kp = crypto::generate_key_pair().expect("generate_key_pair");
    (kp.private_key, kp.public_key)
}

#[test]
fn client_new_parses_keys_and_exposes_accessors() {
    let (priv_key, pub_key) = key_pair();
    let cfg = WaffoConfig::builder()
        .api_key("k")
        .private_key(&priv_key)
        .waffo_public_key(&pub_key)
        .build()
        .unwrap();

    let client = Client::new(cfg).expect("client should build");
    assert_eq!(client.config().api_key, "k");
    // Accessors return the parsed keys (smoke).
    let _ = client.private_key();
    let _ = client.public_key();
}

#[test]
fn client_new_rejects_unparseable_keys() {
    let (priv_key, pub_key) = key_pair();

    let bad_priv = WaffoConfig::builder()
        .api_key("k")
        .private_key("NOT_A_KEY")
        .waffo_public_key(&pub_key)
        .build()
        .unwrap();
    assert!(Client::new(bad_priv).is_err());

    let bad_pub = WaffoConfig::builder()
        .api_key("k")
        .private_key(&priv_key)
        .waffo_public_key("NOT_A_KEY")
        .build()
        .unwrap();
    assert!(Client::new(bad_pub).is_err());
}

#[test]
fn client_with_custom_http_client() {
    let (priv_key, pub_key) = key_pair();
    let cfg = WaffoConfig::builder()
        .api_key("k")
        .private_key(&priv_key)
        .waffo_public_key(&pub_key)
        .build()
        .unwrap();

    let client = Client::with_http_client(cfg, reqwest::Client::new()).expect("client should build");
    let _ = client.public_key();
}
