//! Unit tests for the webhook module: status classification, event helpers,
//! the lenient `FailureReason`, signature verify/parse, signed responses and
//! the axum integration (feature-gated).

use waffo_rs::webhook::{
    self, FailureReason, OrderStatus, PaymentNotificationResult, RefundNotificationResult,
    RefundStatus, SubscriptionChangeNotificationResult, SubscriptionDispatch,
    SubscriptionNotificationResult, SubscriptionStatus, WebhookEvent,
};
use waffo_rs::{Client, WaffoConfig, WaffoError, crypto};

// ---- status classification (From<&str> / as_str) ---------------------------

#[test]
fn order_status_classification_roundtrips() {
    let known = [
        ("PAY_IN_PROGRESS", OrderStatus::PayInProgress),
        ("AUTHORIZATION_REQUIRED", OrderStatus::AuthorizationRequired),
        ("AUTHED_WAITING_CAPTURE", OrderStatus::AuthedWaitingCapture),
        ("PAY_SUCCESS", OrderStatus::PaySuccess),
        ("ORDER_CLOSE", OrderStatus::OrderClose),
    ];
    for (wire, variant) in known {
        assert_eq!(OrderStatus::from(wire), variant);
        assert_eq!(variant.as_str(), wire);
    }
    let other = OrderStatus::from("CAPTURE_IN_PROGRESS");
    assert_eq!(other, OrderStatus::Other("CAPTURE_IN_PROGRESS".to_string()));
    assert_eq!(other.as_str(), "CAPTURE_IN_PROGRESS");
}

#[test]
fn refund_status_classification_roundtrips() {
    let known = [
        ("REFUND_IN_PROGRESS", RefundStatus::InProgress),
        ("ORDER_PARTIALLY_REFUNDED", RefundStatus::PartiallyRefunded),
        ("ORDER_FULLY_REFUNDED", RefundStatus::FullyRefunded),
        ("ORDER_REFUND_FAILED", RefundStatus::Failed),
    ];
    for (wire, variant) in known {
        assert_eq!(RefundStatus::from(wire), variant);
        assert_eq!(variant.as_str(), wire);
    }
    assert_eq!(RefundStatus::from("X").as_str(), "X");
}

#[test]
fn subscription_status_classification_roundtrips() {
    let known = [
        (
            "AUTHORIZATION_REQUIRED",
            SubscriptionStatus::AuthorizationRequired,
        ),
        ("IN_PROGRESS", SubscriptionStatus::InProgress),
        ("ACTIVE", SubscriptionStatus::Active),
        ("CLOSE", SubscriptionStatus::Close),
        ("MERCHANT_CANCELLED", SubscriptionStatus::MerchantCancelled),
        ("USER_CANCELLED", SubscriptionStatus::UserCancelled),
        ("CHANNEL_CANCELLED", SubscriptionStatus::ChannelCancelled),
        ("EXPIRED", SubscriptionStatus::Expired),
    ];
    for (wire, variant) in known {
        assert_eq!(SubscriptionStatus::from(wire), variant);
        assert_eq!(variant.as_str(), wire);
    }
    assert_eq!(SubscriptionStatus::from("Z").as_str(), "Z");
}

// ---- WebhookEvent helpers --------------------------------------------------

fn payment(order_status: &str) -> WebhookEvent {
    WebhookEvent::Payment(PaymentNotificationResult {
        order_status: order_status.to_string(),
        ..Default::default()
    })
}

#[test]
fn event_type_strings() {
    assert_eq!(payment("PAY_SUCCESS").event_type(), "PAYMENT_NOTIFICATION");
    assert_eq!(
        WebhookEvent::Refund(RefundNotificationResult::default()).event_type(),
        "REFUND_NOTIFICATION"
    );
    assert_eq!(
        WebhookEvent::SubscriptionStatus(SubscriptionNotificationResult::default()).event_type(),
        "SUBSCRIPTION_STATUS_NOTIFICATION"
    );
    assert_eq!(
        WebhookEvent::SubscriptionPeriodChanged(SubscriptionNotificationResult::default())
            .event_type(),
        "SUBSCRIPTION_PERIOD_CHANGED_NOTIFICATION"
    );
    assert_eq!(
        WebhookEvent::SubscriptionChange(SubscriptionChangeNotificationResult::default())
            .event_type(),
        "SUBSCRIPTION_CHANGE_NOTIFICATION"
    );
}

#[test]
fn classifiers_only_fire_for_their_event() {
    let pay = payment("PAY_SUCCESS");
    assert_eq!(pay.order_status(), Some(OrderStatus::PaySuccess));
    assert_eq!(pay.refund_status(), None);
    assert_eq!(pay.subscription_status(), None);

    let refund = WebhookEvent::Refund(RefundNotificationResult {
        refund_status: "ORDER_FULLY_REFUNDED".to_string(),
        ..Default::default()
    });
    assert_eq!(refund.refund_status(), Some(RefundStatus::FullyRefunded));
    assert_eq!(refund.order_status(), None);

    let sub = WebhookEvent::SubscriptionStatus(SubscriptionNotificationResult {
        subscription_status: "ACTIVE".to_string(),
        ..Default::default()
    });
    assert_eq!(sub.subscription_status(), Some(SubscriptionStatus::Active));
    assert_eq!(sub.order_status(), None);
}

#[test]
fn subscription_dispatch_status_over_payment() {
    let sub = WebhookEvent::SubscriptionStatus(SubscriptionNotificationResult::default());

    assert!(matches!(
        sub.subscription_dispatch(true, true),
        Some(SubscriptionDispatch::Status(_))
    ));
    assert!(matches!(
        sub.subscription_dispatch(false, true),
        Some(SubscriptionDispatch::Payment(_))
    ));
    assert!(sub.subscription_dispatch(false, false).is_none());

    // Non-status events never dispatch.
    assert!(
        payment("PAY_SUCCESS")
            .subscription_dispatch(true, true)
            .is_none()
    );
}

// ---- FailureReason (lenient deserialize) -----------------------------------

fn failure(json: &str) -> FailureReason {
    serde_json::from_str(json).expect("FailureReason should deserialize")
}

#[test]
fn failure_reason_accepts_object() {
    let fr = failure(r#"{"code":1,"message":"boom"}"#);
    assert!(!fr.is_empty());
    assert_eq!(
        fr.as_map()
            .get("message")
            .and_then(serde_json::Value::as_str),
        Some("boom")
    );
}

#[test]
fn failure_reason_accepts_json_encoded_string() {
    // A JSON string whose contents are themselves a JSON object.
    let fr = failure(r#""{\"code\":7}""#);
    assert_eq!(
        fr.as_map().get("code").and_then(serde_json::Value::as_i64),
        Some(7)
    );
}

#[test]
fn failure_reason_wraps_plain_string_as_message() {
    let fr = failure(r#""something failed""#);
    assert_eq!(
        fr.as_map()
            .get("message")
            .and_then(serde_json::Value::as_str),
        Some("something failed")
    );
}

#[test]
fn failure_reason_empty_and_null_are_empty() {
    assert!(failure("null").is_empty());
    assert!(failure(r#""""#).is_empty());
}

#[test]
fn failure_reason_rejects_unexpected_scalar() {
    assert!(serde_json::from_str::<FailureReason>("42").is_err());
}

#[test]
fn failure_reason_to_json_string() {
    assert_eq!(failure(r#"{"a":1}"#).to_json_string(), r#"{"a":1}"#);
    assert_eq!(failure("null").to_json_string(), "");
}

// ---- verify_and_parse / build_signed_response ------------------------------

/// A client whose `waffo_public_key` matches the returned "Waffo-side" private
/// key, so test webhook bodies can be signed as if by Waffo.
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
fn verify_and_parse_routes_every_event_type() {
    let (client, waffo) = webhook_client();
    let cases = [
        (
            r#"{"eventType":"PAYMENT_NOTIFICATION","result":{"orderStatus":"PAY_SUCCESS"}}"#,
            "PAYMENT_NOTIFICATION",
        ),
        (
            r#"{"eventType":"REFUND_NOTIFICATION","result":{}}"#,
            "REFUND_NOTIFICATION",
        ),
        (
            r#"{"eventType":"SUBSCRIPTION_STATUS_NOTIFICATION","result":{}}"#,
            "SUBSCRIPTION_STATUS_NOTIFICATION",
        ),
        (
            r#"{"eventType":"SUBSCRIPTION_PERIOD_CHANGED_NOTIFICATION","result":{}}"#,
            "SUBSCRIPTION_PERIOD_CHANGED_NOTIFICATION",
        ),
        (
            r#"{"eventType":"SUBSCRIPTION_CHANGE_NOTIFICATION","result":{}}"#,
            "SUBSCRIPTION_CHANGE_NOTIFICATION",
        ),
    ];
    for (body, event_type) in cases {
        let sig = crypto::sign(&waffo, body.as_bytes()).unwrap();
        let event = webhook::verify_and_parse(&client, body.as_bytes(), &sig)
            .expect("a correctly signed webhook should parse");
        assert_eq!(event.event_type(), event_type);
    }
}

#[test]
fn verify_and_parse_decodes_result_fields() {
    let (client, waffo) = webhook_client();
    let body = br#"{"eventType":"PAYMENT_NOTIFICATION","result":{"orderStatus":"PAY_SUCCESS","paymentRequestId":"r-9"}}"#;
    let sig = crypto::sign(&waffo, body).unwrap();
    match webhook::verify_and_parse(&client, body, &sig).unwrap() {
        WebhookEvent::Payment(p) => {
            assert_eq!(p.order_status, "PAY_SUCCESS");
            assert_eq!(p.payment_request_id, "r-9");
        }
        other => panic!("expected a payment event, got {}", other.event_type()),
    }
}

#[test]
fn verify_and_parse_missing_result_defaults() {
    let (client, waffo) = webhook_client();
    let body = br#"{"eventType":"PAYMENT_NOTIFICATION"}"#;
    let sig = crypto::sign(&waffo, body).unwrap();
    match webhook::verify_and_parse(&client, body, &sig).unwrap() {
        WebhookEvent::Payment(p) => assert!(p.order_status.is_empty()),
        _ => panic!("expected a payment event"),
    }
}

#[test]
fn verify_and_parse_rejects_empty_and_wrong_signature() {
    let (client, waffo) = webhook_client();
    let body = br#"{"eventType":"PAYMENT_NOTIFICATION","result":{}}"#;

    // Empty signature.
    assert!(matches!(
        webhook::verify_and_parse(&client, body, ""),
        Err(WaffoError::VerificationFailed)
    ));

    // Signature over different bytes.
    let sig = crypto::sign(&waffo, b"different body").unwrap();
    assert!(matches!(
        webhook::verify_and_parse(&client, body, &sig),
        Err(WaffoError::VerificationFailed)
    ));
}

#[test]
fn verify_and_parse_unknown_event_type() {
    let (client, waffo) = webhook_client();
    let body = br#"{"eventType":"NOPE_NOTIFICATION","result":{}}"#;
    let sig = crypto::sign(&waffo, body).unwrap();
    let err = webhook::verify_and_parse(&client, body, &sig).unwrap_err();
    assert_eq!(err.api_code(), Some("UNKNOWN_EVENT_TYPE"));
}

#[test]
fn build_signed_response_bodies_and_signature() {
    let (client, _waffo) = webhook_client();
    let merchant_pub = client.private_key().to_public_key();

    let (body, sig) = webhook::build_signed_response(&client, true).unwrap();
    assert_eq!(body, r#"{"message":"success"}"#);
    crypto::verify(&merchant_pub, body.as_bytes(), &sig)
        .expect("response signature should verify under the merchant key");

    let (body, sig) = webhook::build_signed_response(&client, false).unwrap();
    assert_eq!(body, r#"{"message":"failed"}"#);
    crypto::verify(&merchant_pub, body.as_bytes(), &sig).unwrap();
}

// ---- a notification result carrying a JSON-encoded failedReason ------------

#[test]
fn notification_decodes_string_encoded_failed_reason() {
    // v1.3.2: sandbox sends failedReason as a JSON-encoded string.
    let json = r#"{"orderStatus":"ORDER_CLOSE","orderFailedReason":"{\"code\":\"E99\"}"}"#;
    let result: PaymentNotificationResult = serde_json::from_str(json).unwrap();
    assert_eq!(result.order_status, "ORDER_CLOSE");
    assert_eq!(
        result
            .order_failed_reason
            .as_map()
            .get("code")
            .and_then(serde_json::Value::as_str),
        Some("E99")
    );
}

// ---- axum integration (feature-gated) --------------------------------------

#[cfg(feature = "axum")]
mod axum_tests {
    use super::{crypto, webhook_client};
    use axum::http::HeaderMap;
    use waffo_rs::webhook::WebhookEvent;
    use waffo_rs::webhook::axum::{
        SIGNATURE_HEADER, parse_request, signature_from_headers, signed_response,
    };

    #[test]
    fn signature_from_headers_reads_the_header() {
        let mut headers = HeaderMap::new();
        assert_eq!(signature_from_headers(&headers), None);
        headers.insert(SIGNATURE_HEADER, "sig-123".parse().unwrap());
        assert_eq!(signature_from_headers(&headers), Some("sig-123"));
    }

    #[test]
    fn signed_response_sets_status_body_and_header() {
        let (client, _waffo) = webhook_client();

        let ok = signed_response(&client, true);
        assert_eq!(ok.status().as_u16(), 200);
        assert!(ok.headers().get(SIGNATURE_HEADER).is_some());

        let failed = signed_response(&client, false);
        assert_eq!(failed.status().as_u16(), 400);
    }

    #[test]
    fn parse_request_pulls_signature_and_verifies() {
        let (client, waffo) = webhook_client();
        let body = br#"{"eventType":"PAYMENT_NOTIFICATION","result":{}}"#;
        let sig = crypto::sign(&waffo, body).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(SIGNATURE_HEADER, sig.parse().unwrap());

        let event = parse_request(&client, &headers, body).unwrap();
        assert!(matches!(event, WebhookEvent::Payment(_)));
    }
}
