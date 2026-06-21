//! Transport-path tests for the uniform `send` pipeline, driven through a
//! mocked Waffo API server (`wiremock`). Exercises request signing/headers,
//! response signature verification, business-error / unknown-status mapping,
//! the read/write transport-failure split, and per-request options.

use waffo_rs::biz::order;
use waffo_rs::{Client, RequestOptions, WaffoConfig, WaffoError, crypto};
use wiremock::matchers::{header, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Build a client pointed at `base_url`, plus the "Waffo-side" private key so
/// tests can sign mock response bodies the client will accept.
fn client_for(base_url: &str) -> (Client, rsa::RsaPrivateKey) {
    let merchant = crypto::generate_key_pair().unwrap();
    let waffo = crypto::generate_key_pair().unwrap();
    let cfg = WaffoConfig::builder()
        .api_key("test-key")
        .private_key(&merchant.private_key)
        .waffo_public_key(&waffo.public_key)
        .merchant_id("M-TEST")
        .base_url(base_url)
        .build()
        .unwrap();
    let client = Client::new(cfg).unwrap();
    let waffo_priv = crypto::parse_private_key(&waffo.private_key).unwrap();
    (client, waffo_priv)
}

const OK_BODY: &str = r#"{"code":"0","data":{}}"#;

#[tokio::test]
async fn request_carries_api_key_version_and_signature_headers() {
    let server = MockServer::start().await;
    // The mock only matches when the expected headers are present, so a
    // successful call proves the request was signed and labeled correctly.
    Mock::given(method("POST"))
        .and(path("/order/create"))
        .and(header("X-API-KEY", "test-key"))
        .and(header("X-API-VERSION", "1.0.0"))
        .and(header_exists("X-SIGNATURE"))
        .and(header_exists("X-SDK-VERSION"))
        .respond_with(ResponseTemplate::new(200).set_body_string(OK_BODY))
        .mount(&server)
        .await;

    let (client, _waffo) = client_for(&server.uri());
    let resp = order::create(&client, order::CreateOrderParams::default(), None).await;
    assert!(
        resp.is_ok(),
        "signed request should match the mock: {resp:?}"
    );
}

#[tokio::test]
async fn accepts_a_validly_signed_response() {
    let server = MockServer::start().await;
    let (client, waffo_priv) = client_for(&server.uri());
    let signature = crypto::sign(&waffo_priv, OK_BODY.as_bytes()).unwrap();

    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("X-SIGNATURE", signature.as_str())
                .set_body_string(OK_BODY),
        )
        .mount(&server)
        .await;

    let resp = order::create(&client, order::CreateOrderParams::default(), None).await;
    assert!(
        resp.is_ok(),
        "a validly signed response should be accepted: {resp:?}"
    );
}

#[tokio::test]
async fn rejects_an_invalidly_signed_response() {
    let server = MockServer::start().await;
    let (client, _waffo) = client_for(&server.uri());

    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("X-SIGNATURE", "not-a-real-signature")
                .set_body_string(OK_BODY),
        )
        .mount(&server)
        .await;

    let err = order::create(&client, order::CreateOrderParams::default(), None)
        .await
        .unwrap_err();
    assert!(matches!(err, WaffoError::VerificationFailed));
}

#[tokio::test]
async fn maps_business_error_code() {
    let server = MockServer::start().await;
    let (client, _waffo) = client_for(&server.uri());

    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"code":"A0014","msg":"rejected"}"#),
        )
        .mount(&server)
        .await;

    let err = order::create(&client, order::CreateOrderParams::default(), None)
        .await
        .unwrap_err();
    assert_eq!(err.api_code(), Some("A0014"));
    assert!(!err.is_unknown_status());
}

#[tokio::test]
async fn maps_e0001_to_unknown_status() {
    let server = MockServer::start().await;
    let (client, _waffo) = client_for(&server.uri());

    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"code":"E0001","msg":"uncertain"}"#),
        )
        .mount(&server)
        .await;

    let err = order::create(&client, order::CreateOrderParams::default(), None)
        .await
        .unwrap_err();
    assert!(err.is_unknown_status());
    assert_eq!(err.api_code(), Some("E0001"));
}

#[tokio::test]
async fn read_endpoint_transport_failure_folds_to_unknown_status() {
    // Nothing listens on this port -> connection refused.
    let (client, _waffo) = client_for("http://127.0.0.1:9");
    let err = order::inquiry(&client, order::InquiryOrderParams::default(), None)
        .await
        .unwrap_err();
    assert!(
        err.is_unknown_status(),
        "a read-endpoint transport failure should fold to unknown status: {err:?}"
    );
}

#[tokio::test]
async fn write_endpoint_transport_failure_surfaces_as_transport_error() {
    let (client, _waffo) = client_for("http://127.0.0.1:9");
    let err = order::create(&client, order::CreateOrderParams::default(), None)
        .await
        .unwrap_err();
    assert!(
        matches!(err, WaffoError::Transport(_)),
        "a write-endpoint transport failure should surface as a transport error: {err:?}"
    );
}

#[tokio::test]
async fn per_request_options_add_headers() {
    let server = MockServer::start().await;
    let (client, _waffo) = client_for(&server.uri());

    Mock::given(method("POST"))
        .and(header("X-Test", "yes"))
        .respond_with(ResponseTemplate::new(200).set_body_string(OK_BODY))
        .mount(&server)
        .await;

    let opts = RequestOptions {
        headers: vec![("X-Test".to_string(), "yes".to_string())],
        ..Default::default()
    };
    let resp = order::create(&client, order::CreateOrderParams::default(), Some(&opts)).await;
    assert!(
        resp.is_ok(),
        "the per-request header should reach the server: {resp:?}"
    );
}
