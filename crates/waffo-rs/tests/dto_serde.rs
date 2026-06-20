//! DTO wire-fidelity round-trips. These assert that the Rust DTOs serialize
//! under the exact camelCase JSON keys the Go SDK uses (this is a payment SDK;
//! byte-for-byte wire fidelity is required), and that response `*Data` types
//! preserve unknown server fields through their flattened `extra` map.
//!
//! Types come from the sibling-authored modules:
//!   waffo_rs::biz::{order, subscription, merchant}
//!   waffo_rs::webhook
//! Field/JSON names are taken from the authoritative Go source:
//!   waffo-go/types/{order,subscription,merchant}/*.go and
//!   waffo-go/core/webhook_handler.go

use serde_json::Value;

use waffo_rs::biz::merchant;
use waffo_rs::biz::order;
use waffo_rs::biz::subscription;

/// Serialize `value` and return its parsed JSON object.
fn to_object<T: serde::Serialize>(value: &T) -> serde_json::Map<String, Value> {
    let s = serde_json::to_string(value).expect("serialize failed");
    let v: Value = serde_json::from_str(&s).expect("re-parse failed");
    v.as_object().expect("expected a JSON object").clone()
}

/// Assert every key in `keys` is present in `obj`.
fn assert_has_keys(obj: &serde_json::Map<String, Value>, keys: &[&str]) {
    for k in keys {
        assert!(
            obj.contains_key(*k),
            "expected JSON key {k:?}; got keys: {:?}",
            obj.keys().collect::<Vec<_>>()
        );
    }
}

// ---- order::CreateOrderParams ----------------------------------------------

#[test]
fn create_order_params_wire_keys() {
    let params = order::CreateOrderParams {
        payment_request_id: "pr-123".to_string(),
        merchant_order_id: "mo-456".to_string(),
        order_currency: "USD".to_string(),
        order_amount: "100.00".to_string(),
        order_description: "Test order".to_string(),
        notify_url: "https://example.com/notify".to_string(),
        ..Default::default()
    };

    let obj = to_object(&params);
    // Required (non-omitempty) keys must always be present and camelCase.
    assert_has_keys(
        &obj,
        &[
            "paymentRequestId",
            "merchantOrderId",
            "orderCurrency",
            "orderAmount",
            "orderDescription",
            "notifyUrl",
        ],
    );
    assert_eq!(obj["paymentRequestId"], Value::from("pr-123"));
    assert_eq!(obj["orderAmount"], Value::from("100.00"));
}

// ---- order::CreateOrderData (response) -------------------------------------

#[test]
fn create_order_data_wire_keys_and_forward_compat() {
    // Sample server JSON, including an unknown forward-compat field.
    let server = r#"{
        "paymentRequestId": "pr-123",
        "merchantOrderId": "mo-456",
        "acquiringOrderId": "aq-789",
        "orderStatus": "PAY_SUCCESS",
        "orderAction": "",
        "someBrandNewField": {"nested": true}
    }"#;

    let data: order::CreateOrderData =
        serde_json::from_str(server).expect("deserialize CreateOrderData failed");

    // Round-trips with camelCase keys preserved.
    let obj = to_object(&data);
    assert_has_keys(
        &obj,
        &[
            "paymentRequestId",
            "merchantOrderId",
            "acquiringOrderId",
            "orderStatus",
        ],
    );
    assert_eq!(obj["acquiringOrderId"], Value::from("aq-789"));
    assert_eq!(obj["orderStatus"], Value::from("PAY_SUCCESS"));

    // The unknown field is preserved via the flattened `extra` map and survives
    // a re-serialize (forward compatibility).
    assert!(
        obj.contains_key("someBrandNewField"),
        "unknown server field should be preserved through the flattened extra map"
    );
}

// ---- order::InquiryOrderData (response) ------------------------------------

#[test]
fn inquiry_order_data_forward_compat() {
    let server = r#"{
        "paymentRequestId": "pr-1",
        "merchantOrderId": "mo-1",
        "acquiringOrderId": "aq-1",
        "orderStatus": "PAY_SUCCESS",
        "orderCurrency": "USD",
        "orderAmount": "100.00",
        "futureOnlyField": "keep-me"
    }"#;

    let data: order::InquiryOrderData =
        serde_json::from_str(server).expect("deserialize InquiryOrderData failed");

    let obj = to_object(&data);
    assert_has_keys(
        &obj,
        &[
            "paymentRequestId",
            "merchantOrderId",
            "orderStatus",
            "orderCurrency",
            "orderAmount",
        ],
    );
    assert_eq!(
        obj.get("futureOnlyField"),
        Some(&Value::from("keep-me")),
        "unknown server field should round-trip through extra"
    );
}

// ---- order::RefundOrderParams: extra_params injection ----------------------

#[test]
fn refund_order_params_extra_params_serializes_under_extra_params_key() {
    let mut extra = waffo_rs::ExtraParams::new();
    extra.insert("customKey".to_string(), Value::from("customValue"));

    let params = order::RefundOrderParams {
        refund_request_id: "rr-1".to_string(),
        acquiring_order_id: "aq-1".to_string(),
        refund_amount: "10.00".to_string(),
        refund_reason: "duplicate".to_string(),
        extra_params: Some(extra),
        ..Default::default()
    };

    let obj = to_object(&params);
    assert_has_keys(
        &obj,
        &[
            "refundRequestId",
            "acquiringOrderId",
            "refundAmount",
            "refundReason",
            "extraParams",
        ],
    );
    // The escape hatch serializes under the exact "extraParams" wire key.
    let ep = obj["extraParams"]
        .as_object()
        .expect("extraParams should be an object");
    assert_eq!(ep.get("customKey"), Some(&Value::from("customValue")));
}

// ---- subscription::CreateSubscriptionParams --------------------------------

#[test]
fn create_subscription_params_wire_keys() {
    let params = subscription::CreateSubscriptionParams {
        subscription_request: "sub-req-1".to_string(),
        merchant_subscription_id: "msid-1".to_string(),
        currency: "USD".to_string(),
        amount: "9.99".to_string(),
        notify_url: "https://example.com/notify".to_string(),
        ..Default::default()
    };

    let obj = to_object(&params);
    assert_has_keys(
        &obj,
        &[
            "subscriptionRequest",
            "merchantSubscriptionId",
            "currency",
            "amount",
            "notifyUrl",
        ],
    );
    assert_eq!(obj["subscriptionRequest"], Value::from("sub-req-1"));
    assert_eq!(obj["amount"], Value::from("9.99"));
}

// ---- subscription::InquirySubscriptionData (response) ----------------------

#[test]
fn inquiry_subscription_data_forward_compat() {
    let server = r#"{
        "subscriptionRequest": "sub-req-1",
        "merchantSubscriptionId": "msid-1",
        "subscriptionId": "sid-1",
        "subscriptionStatus": "ACTIVE",
        "currency": "USD",
        "amount": "9.99",
        "brandNewServerField": [1, 2, 3]
    }"#;

    let data: subscription::InquirySubscriptionData =
        serde_json::from_str(server).expect("deserialize InquirySubscriptionData failed");

    let obj = to_object(&data);
    assert_has_keys(
        &obj,
        &[
            "subscriptionRequest",
            "merchantSubscriptionId",
            "subscriptionId",
            "subscriptionStatus",
        ],
    );
    assert!(
        obj.contains_key("brandNewServerField"),
        "unknown server field should be preserved through the flattened extra map"
    );
}

// ---- merchant::InquiryMerchantConfigData (response) ------------------------

#[test]
fn inquiry_merchant_config_data_forward_compat() {
    let server = r#"{
        "merchantId": "m-1",
        "totalDailyLimit": {"USD": "10000.00"},
        "newServerOnlyField": "preserved"
    }"#;

    let data: merchant::InquiryMerchantConfigData =
        serde_json::from_str(server).expect("deserialize InquiryMerchantConfigData failed");

    let obj = to_object(&data);
    assert_has_keys(&obj, &["merchantId"]);
    assert_eq!(obj["merchantId"], Value::from("m-1"));
    assert_eq!(
        obj.get("newServerOnlyField"),
        Some(&Value::from("preserved")),
        "unknown server field should round-trip through extra"
    );
}

// ---- merchant::InquiryMerchantConfigParams ---------------------------------

#[test]
fn inquiry_merchant_config_params_wire_keys() {
    // merchantId on the merchant-config request is a required (non-omitempty)
    // field: a plain String, always present.
    let params = merchant::InquiryMerchantConfigParams {
        merchant_id: "m-1".to_string(),
        ..Default::default()
    };

    let obj = to_object(&params);
    assert_has_keys(&obj, &["merchantId"]);
    assert_eq!(obj["merchantId"], Value::from("m-1"));
}

// ---- webhook: PaymentNotificationResult forward-compat ---------------------

#[test]
fn webhook_payment_result_forward_compat() {
    let server = r#"{
        "paymentRequestId": "pr-1",
        "merchantOrderId": "mo-1",
        "acquiringOrderId": "aq-1",
        "orderStatus": "PAY_SUCCESS",
        "orderCurrency": "USD",
        "orderAmount": "100.00",
        "brandNewWebhookField": "kept"
    }"#;

    let result: waffo_rs::webhook::PaymentNotificationResult =
        serde_json::from_str(server).expect("deserialize PaymentNotificationResult failed");

    let obj = to_object(&result);
    assert_has_keys(
        &obj,
        &[
            "paymentRequestId",
            "merchantOrderId",
            "acquiringOrderId",
            "orderStatus",
        ],
    );
    assert!(
        obj.contains_key("brandNewWebhookField"),
        "unknown webhook field should be preserved through the flattened extra map"
    );
}
