//! Tests for the chargeback business domain: status/phase classification,
//! `ChargebackOrder` (de)serialization + null tolerance, the four endpoints
//! over a mocked Waffo API server (`wiremock`), and the `CHARGEBACK_NOTIFICATION`
//! webhook routing.

use waffo_rs::biz::chargeback::{
    self, ChargebackEvidence, ChargebackMessage, ChargebackOrder, ChargebackPhase, ChargebackStatus,
};
use waffo_rs::webhook::{self, WebhookEvent};
use waffo_rs::{Client, WaffoConfig, WaffoError, crypto};
use wiremock::matchers::{body_string_contains, header, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// classification (From<&str> / as_str / is_final)
// ---------------------------------------------------------------------------

#[test]
fn phase_classification_roundtrips() {
    let known = [
        ("NEW", ChargebackPhase::New),
        ("RETURNED", ChargebackPhase::Returned),
        ("PROCESSING", ChargebackPhase::Processing),
        ("FINAL", ChargebackPhase::Final),
    ];
    for (wire, variant) in known {
        assert_eq!(ChargebackPhase::from(wire), variant);
        assert_eq!(variant.as_str(), wire);
    }
    let other = ChargebackPhase::from("WHATEVER");
    assert_eq!(other, ChargebackPhase::Other("WHATEVER".to_string()));
    assert_eq!(other.as_str(), "WHATEVER");
}

#[test]
fn status_classification_roundtrips() {
    let known = [
        ("EVIDENCE_REQUIRED", ChargebackStatus::EvidenceRequired),
        ("UNDER_REVIEW", ChargebackStatus::UnderReview),
        ("ACCEPTED", ChargebackStatus::Accepted),
        ("CANCELED", ChargebackStatus::Canceled),
        ("CASE_LOST", ChargebackStatus::CaseLost),
        ("CASE_WON", ChargebackStatus::CaseWon),
        ("EXPIRED", ChargebackStatus::Expired),
        ("SETTLED", ChargebackStatus::Settled),
    ];
    for (wire, variant) in known {
        assert_eq!(ChargebackStatus::from(wire), variant);
        assert_eq!(variant.as_str(), wire);
    }
    assert_eq!(ChargebackStatus::from("NOPE").as_str(), "NOPE");
}

#[test]
fn status_is_final() {
    for s in [
        ChargebackStatus::Accepted,
        ChargebackStatus::Canceled,
        ChargebackStatus::CaseLost,
        ChargebackStatus::CaseWon,
        ChargebackStatus::Expired,
        ChargebackStatus::Settled,
    ] {
        assert!(s.is_final(), "{s:?} should be final");
    }
    for s in [
        ChargebackStatus::EvidenceRequired,
        ChargebackStatus::UnderReview,
        ChargebackStatus::Other("X".to_string()),
    ] {
        assert!(!s.is_final(), "{s:?} should not be final");
    }
}

// ---------------------------------------------------------------------------
// (de)serialization
// ---------------------------------------------------------------------------

const FULL_ORDER: &str = r#"{
    "chargebackId": "CB-1",
    "chargebackPhase": "NEW",
    "originalOrderId": "O-1",
    "merchantId": "M-1",
    "chargebackStatus": "EVIDENCE_REQUIRED",
    "amount": "2.27",
    "currency": "IDR",
    "feeAmount": "0.50",
    "feeCurrency": "USD",
    "reasonCode": "10.4",
    "reason": "Fraudulent transaction",
    "description": "cardholder dispute",
    "chargebackDateTime": "2025-12-01T03:00:00.000Z",
    "expiryDateTime": "2025-12-08T03:00:00.000Z",
    "message": {"messageId": "MSG-1", "notes": "please review", "documents": "f1,f2"},
    "evidence": {"othersText": "see attached", "othersFile": "f3"},
    "serverAddedField": "ignored-but-kept"
}"#;

#[test]
fn order_deserializes_all_fields() {
    let o: ChargebackOrder = serde_json::from_str(FULL_ORDER).unwrap();
    assert_eq!(o.chargeback_id, "CB-1");
    assert_eq!(o.phase(), ChargebackPhase::New);
    assert_eq!(o.status(), ChargebackStatus::EvidenceRequired);
    assert_eq!(o.fee_amount, "0.50");
    assert_eq!(o.reason_code, "10.4");

    let msg = o.message.as_ref().expect("message present");
    assert_eq!(msg.message_id, "MSG-1");
    assert_eq!(msg.documents, "f1,f2");
    let ev = o.evidence.as_ref().expect("evidence present");
    assert_eq!(ev.others_file, "f3");

    // forward-compat catch-all keeps unknown fields.
    assert_eq!(
        o.extra.get("serverAddedField").and_then(|v| v.as_str()),
        Some("ignored-but-kept")
    );
}

#[test]
fn order_tolerates_null_collections_and_objects() {
    // The sandbox returns `null` for absent objects/strings; this must not error
    // (regression for the `null_as_default` helper).
    let json = r#"{
        "chargebackId": "CB-2",
        "chargebackStatus": null,
        "description": null,
        "message": null,
        "evidence": null
    }"#;
    let o: ChargebackOrder = serde_json::from_str(json).unwrap();
    assert_eq!(o.chargeback_id, "CB-2");
    assert_eq!(o.chargeback_status, "");
    assert_eq!(o.status(), ChargebackStatus::Other(String::new()));
    assert!(o.message.is_none());
    assert!(o.evidence.is_none());
}

#[test]
fn list_data_tolerates_null_records() {
    let json = r#"{"total": 0, "size": 20, "current": 1, "pages": 0, "records": null}"#;
    let data: chargeback::ChargebackListData = serde_json::from_str(json).unwrap();
    assert_eq!(data.size, 20);
    assert!(data.records.is_empty());
}

// ---------------------------------------------------------------------------
// endpoints over a mocked server
// ---------------------------------------------------------------------------

/// Build a client pointed at `base_url`; `merchant_id` is set so injection can
/// fill it into request params that leave it empty.
fn client_for(base_url: &str) -> Client {
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
    Client::new(cfg).unwrap()
}

fn ok_envelope(data: &str) -> String {
    format!(r#"{{"code":"0","data":{data}}}"#)
}

#[tokio::test]
async fn inquiry_hits_path_signs_and_injects_merchant_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chargeback/inquiry"))
        .and(header("X-API-KEY", "test-key"))
        .and(header_exists("X-SIGNATURE"))
        // merchant_id left empty in params -> injected from config.
        .and(body_string_contains("M-TEST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(ok_envelope(FULL_ORDER)))
        .mount(&server)
        .await;

    let client = client_for(&server.uri());
    let order = chargeback::inquiry(
        &client,
        chargeback::InquiryChargebackParams {
            chargeback_id: Some("CB-1".to_string()),
            ..Default::default()
        },
        None,
    )
    .await
    .expect("inquiry should succeed");
    assert_eq!(order.chargeback_id, "CB-1");
    assert_eq!(order.status(), ChargebackStatus::EvidenceRequired);
}

#[tokio::test]
async fn update_submits_evidence() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chargeback/update"))
        .and(body_string_contains("see attached"))
        .respond_with(ResponseTemplate::new(200).set_body_string(ok_envelope(FULL_ORDER)))
        .mount(&server)
        .await;

    let client = client_for(&server.uri());
    let order = chargeback::update(
        &client,
        chargeback::UpdateChargebackParams {
            chargeback_id: "CB-1".to_string(),
            message: Some(ChargebackMessage {
                notes: "rebuttal".to_string(),
                documents: "f1".to_string(),
                ..Default::default()
            }),
            evidence: Some(ChargebackEvidence {
                others_text: "see attached".to_string(),
                others_file: "f3".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        },
        None,
    )
    .await
    .expect("update should succeed");
    assert_eq!(order.chargeback_id, "CB-1");
}

#[tokio::test]
async fn accept_hits_accept_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chargeback/accept"))
        .respond_with(ResponseTemplate::new(200).set_body_string(ok_envelope(FULL_ORDER)))
        .mount(&server)
        .await;

    let client = client_for(&server.uri());
    let order = chargeback::accept(
        &client,
        chargeback::AcceptChargebackParams {
            chargeback_id: "CB-1".to_string(),
            ..Default::default()
        },
        None,
    )
    .await
    .expect("accept should succeed");
    assert_eq!(order.chargeback_id, "CB-1");
}

#[tokio::test]
async fn list_returns_a_page() {
    let server = MockServer::start().await;
    let page = format!(r#"{{"total":1,"size":20,"current":1,"pages":1,"records":[{FULL_ORDER}]}}"#);
    Mock::given(method("POST"))
        .and(path("/chargeback/list"))
        .respond_with(ResponseTemplate::new(200).set_body_string(ok_envelope(&page)))
        .mount(&server)
        .await;

    let client = client_for(&server.uri());
    let data = chargeback::list(
        &client,
        chargeback::ListChargebackParams {
            chargeback_status: Some(vec!["EVIDENCE_REQUIRED".to_string()]),
            start_time: "2025-12-01T00:00:00+08:00".to_string(),
            end_time: "2025-12-31T00:00:00+08:00".to_string(),
            page_num: 1,
            page_size: 20,
            ..Default::default()
        },
        None,
    )
    .await
    .expect("list should succeed");
    assert_eq!(data.total, 1);
    assert_eq!(data.records.len(), 1);
    assert_eq!(data.records[0].chargeback_id, "CB-1");
}

#[tokio::test]
async fn maps_chargeback_business_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"code":"A0042","msg":"already in final status"}"#),
        )
        .mount(&server)
        .await;

    let client = client_for(&server.uri());
    let err = chargeback::accept(
        &client,
        chargeback::AcceptChargebackParams {
            chargeback_id: "CB-1".to_string(),
            ..Default::default()
        },
        None,
    )
    .await
    .unwrap_err();
    assert_eq!(err.api_code(), Some("A0042"));
    assert!(!err.is_unknown_status());
}

#[tokio::test]
async fn inquiry_maps_e0001_to_unknown_status() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"code":"E0001","msg":"?"}"#))
        .mount(&server)
        .await;

    let client = client_for(&server.uri());
    let err = chargeback::inquiry(
        &client,
        chargeback::InquiryChargebackParams {
            chargeback_id: Some("CB-1".to_string()),
            ..Default::default()
        },
        None,
    )
    .await
    .unwrap_err();
    assert!(err.is_unknown_status());
}

// ---------------------------------------------------------------------------
// webhook routing
// ---------------------------------------------------------------------------

/// A client whose `waffo_public_key` matches the returned private key, so test
/// webhook bodies can be signed as if by Waffo.
fn webhook_client() -> (Client, rsa::RsaPrivateKey) {
    let merchant = crypto::generate_key_pair().unwrap();
    let waffo = crypto::generate_key_pair().unwrap();
    let cfg = WaffoConfig::builder()
        .api_key("k")
        .private_key(&merchant.private_key)
        .waffo_public_key(&waffo.public_key)
        .build()
        .unwrap();
    let client = Client::new(cfg).unwrap();
    let waffo_priv = crypto::parse_private_key(&waffo.private_key).unwrap();
    (client, waffo_priv)
}

#[test]
fn verify_and_parse_routes_chargeback_event() {
    let (client, waffo) = webhook_client();
    let body = br#"{"eventType":"CHARGEBACK_NOTIFICATION","result":{"chargebackId":"CB-9","chargebackPhase":"FINAL","chargebackStatus":"CASE_LOST"}}"#;
    let sig = crypto::sign(&waffo, body).unwrap();

    let event = webhook::verify_and_parse(&client, body, &sig).expect("should parse");
    assert_eq!(event.event_type(), "CHARGEBACK_NOTIFICATION");
    assert_eq!(event.chargeback_status(), Some(ChargebackStatus::CaseLost));
    assert_eq!(event.chargeback_phase(), Some(ChargebackPhase::Final));
    // chargeback classifiers don't fire for other events, and vice versa.
    assert_eq!(event.order_status(), None);

    match event {
        WebhookEvent::Chargeback(c) => {
            assert_eq!(c.chargeback_id, "CB-9");
            assert!(c.status().is_final());
        }
        other => panic!("expected a chargeback event, got {}", other.event_type()),
    }
}

#[test]
fn verify_and_parse_rejects_bad_chargeback_signature() {
    let (client, waffo) = webhook_client();
    let body = br#"{"eventType":"CHARGEBACK_NOTIFICATION","result":{}}"#;
    let sig = crypto::sign(&waffo, b"different bytes").unwrap();
    assert!(matches!(
        webhook::verify_and_parse(&client, body, &sig),
        Err(WaffoError::VerificationFailed)
    ));
}
