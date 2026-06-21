//! Regression tests: the Waffo sandbox returns an explicit JSON `null` for some
//! empty collection fields. Deserialization must treat that as the default
//! (empty), not fail — see `common::de::null_as_default`. (The merchant-config
//! case was caught by the live e2e run.)

use waffo_rs::biz::{merchant, subscription};
use waffo_rs::webhook::PaymentNotificationResult;

#[test]
fn merchant_config_tolerates_null_limit_maps() {
    let json = r#"{
        "merchantId": "M1",
        "totalDailyLimit": null,
        "remainingDailyLimit": null,
        "transactionLimit": null
    }"#;
    let data: merchant::InquiryMerchantConfigData = serde_json::from_str(json).unwrap();
    assert_eq!(data.merchant_id.as_deref(), Some("M1"));
    assert!(data.total_daily_limit.is_empty());
    assert!(data.remaining_daily_limit.is_empty());
    assert!(data.transaction_limit.is_empty());
}

#[test]
fn pay_method_config_tolerates_null_details() {
    let json = r#"{"merchantId":"M1","payMethodDetails":null}"#;
    let data: merchant::InquiryPayMethodConfigData = serde_json::from_str(json).unwrap();
    assert_eq!(data.merchant_id.as_deref(), Some("M1"));
    assert!(data.pay_method_details.is_empty());
}

#[test]
fn subscription_inquiry_tolerates_null_payment_details() {
    let json = r#"{"subscriptionId":"s1","paymentDetails":null}"#;
    let data: subscription::InquirySubscriptionData = serde_json::from_str(json).unwrap();
    assert_eq!(data.subscription_id.as_deref(), Some("s1"));
    assert!(data.payment_details.is_empty());
}

#[test]
fn webhook_payment_tolerates_null_info_maps() {
    let json =
        r#"{"orderStatus":"PAY_SUCCESS","merchantInfo":null,"userInfo":null,"paymentInfo":null}"#;
    let result: PaymentNotificationResult = serde_json::from_str(json).unwrap();
    assert_eq!(result.order_status, "PAY_SUCCESS");
    assert!(result.merchant_info.is_empty());
    assert!(result.user_info.is_empty());
    assert!(result.payment_info.is_empty());
}
