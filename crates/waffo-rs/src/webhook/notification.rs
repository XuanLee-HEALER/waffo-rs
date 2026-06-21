//! Webhook notification result DTOs and the lenient [`FailureReason`] type.
//!
//! These are the `result` payloads carried inside the `{eventType, result}`
//! webhook envelope. Field names and JSON tags mirror the Go SDK
//! (`core/webhook_handler.go`) byte-for-byte — this is a payment SDK, wire
//! fidelity is critical. Every result struct carries a `#[serde(flatten)]`
//! forward-compat catch-all so deserialization never fails on new fields.

use std::collections::HashMap;

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

/// Lenient failure-reason payload (Go `FailureReason map[string]interface{}`).
///
/// Some sandbox callbacks send the failure-reason fields as a JSON-encoded
/// *string* instead of an object. This type stays map-like and its custom
/// [`Deserialize`] accepts every wire shape the Go `UnmarshalJSON` did:
///
/// - a JSON object `{...}`;
/// - a JSON-encoded object string `"{...}"`;
/// - a plain string `"oops"` → wrapped as `{"message": "oops"}`;
/// - `null` / empty → an empty map.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(transparent)]
pub struct FailureReason(pub serde_json::Map<String, serde_json::Value>);

impl FailureReason {
    /// True when there is no failure information.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Borrow the underlying map.
    pub fn as_map(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.0
    }

    /// The failure reason serialized as a JSON object string, or `""` when empty
    /// (mirrors Go `FailureReason.String()`).
    pub fn to_json_string(&self) -> String {
        if self.0.is_empty() {
            return String::new();
        }
        serde_json::to_string(&self.0).unwrap_or_default()
    }
}

impl<'de> Deserialize<'de> for FailureReason {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Accept any JSON value first, then interpret it leniently — matching
        // Go's `UnmarshalJSON`, which tries object, then string-encoded object,
        // then plain string.
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            // null / empty -> empty map
            serde_json::Value::Null => Ok(FailureReason(serde_json::Map::new())),
            // object -> use directly
            serde_json::Value::Object(map) => Ok(FailureReason(map)),
            // string -> either a JSON-encoded object, "" (empty), or plain text
            serde_json::Value::String(raw) => {
                if raw.is_empty() {
                    return Ok(FailureReason(serde_json::Map::new()));
                }
                if let Ok(serde_json::Value::Object(map)) =
                    serde_json::from_str::<serde_json::Value>(&raw)
                {
                    return Ok(FailureReason(map));
                }
                let mut map = serde_json::Map::new();
                map.insert("message".to_string(), serde_json::Value::String(raw));
                Ok(FailureReason(map))
            }
            // any other scalar (number/bool/array) -> not expected; reject so
            // callers see a clear error rather than silent data loss.
            other => Err(de::Error::custom(format!(
                "failureReason: unexpected JSON value: {other}"
            ))),
        }
    }
}

/// Result data of a `PAYMENT_NOTIFICATION` webhook.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaymentNotificationResult {
    #[serde(
        rename = "paymentRequestId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub payment_request_id: String,
    #[serde(
        rename = "merchantOrderId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub merchant_order_id: String,
    #[serde(
        rename = "acquiringOrderId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub acquiring_order_id: String,
    #[serde(
        rename = "orderStatus",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub order_status: String,
    #[serde(
        rename = "orderAction",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub order_action: String,
    #[serde(
        rename = "orderCurrency",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub order_currency: String,
    #[serde(
        rename = "orderAmount",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub order_amount: String,
    #[serde(
        rename = "userCurrency",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub user_currency: String,
    #[serde(
        rename = "finalDealAmount",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub final_deal_amount: String,
    #[serde(
        rename = "orderDescription",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub order_description: String,
    #[serde(
        rename = "merchantInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub merchant_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "userInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub user_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "goodsInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub goods_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "addressInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub address_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "paymentInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub payment_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "orderRequestedAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub order_requested_at: String,
    #[serde(
        rename = "orderExpiredAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub order_expired_at: String,
    #[serde(
        rename = "orderUpdatedAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub order_updated_at: String,
    #[serde(
        rename = "orderCompletedAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub order_completed_at: String,
    #[serde(
        rename = "orderFailedReason",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "FailureReason::is_empty"
    )]
    pub order_failed_reason: FailureReason,
    #[serde(
        rename = "extendInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub extend_info: String,
    #[serde(
        rename = "subscriptionInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_info: Option<crate::biz::subscription::SubscriptionInfo>,
    #[serde(
        rename = "refundExpiryAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub refund_expiry_at: String,
    #[serde(
        rename = "cancelRedirectUrl",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub cancel_redirect_url: String,

    /// Forward-compat catch-all for server fields the SDK does not yet model.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Result data of a `REFUND_NOTIFICATION` webhook.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RefundNotificationResult {
    #[serde(
        rename = "refundRequestId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub refund_request_id: String,
    #[serde(
        rename = "merchantRefundOrderId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub merchant_refund_order_id: String,
    #[serde(
        rename = "acquiringOrderId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub acquiring_order_id: String,
    #[serde(
        rename = "acquiringRefundOrderId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub acquiring_refund_order_id: String,
    #[serde(
        rename = "origPaymentRequestId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub orig_payment_request_id: String,
    #[serde(
        rename = "refundAmount",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub refund_amount: String,
    #[serde(
        rename = "refundStatus",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub refund_status: String,
    #[serde(
        rename = "remainingRefundAmount",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub remaining_refund_amount: String,
    #[serde(
        rename = "userCurrency",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub user_currency: String,
    #[serde(
        rename = "finalDealAmount",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub final_deal_amount: String,
    #[serde(
        rename = "refundReason",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub refund_reason: String,
    #[serde(
        rename = "refundRequestedAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub refund_requested_at: String,
    #[serde(
        rename = "refundUpdatedAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub refund_updated_at: String,
    #[serde(
        rename = "refundCompletedAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub refund_completed_at: String,
    #[serde(
        rename = "refundFailedReason",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "FailureReason::is_empty"
    )]
    pub refund_failed_reason: FailureReason,
    #[serde(
        rename = "userInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub user_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "refundSource",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub refund_source: String,
    #[serde(
        rename = "extendInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub extend_info: String,

    /// Forward-compat catch-all for server fields the SDK does not yet model.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Result data of a `SUBSCRIPTION_STATUS_NOTIFICATION` or
/// `SUBSCRIPTION_PERIOD_CHANGED_NOTIFICATION` webhook (shared shape).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionNotificationResult {
    #[serde(
        rename = "subscriptionRequest",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub subscription_request: String,
    #[serde(
        rename = "merchantSubscriptionId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub merchant_subscription_id: String,
    #[serde(
        rename = "subscriptionId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub subscription_id: String,
    #[serde(
        rename = "payMethodSubscriptionId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub pay_method_subscription_id: String,
    #[serde(
        rename = "subscriptionStatus",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub subscription_status: String,
    #[serde(
        rename = "subscriptionAction",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub subscription_action: String,
    #[serde(
        rename = "currency",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub currency: String,
    #[serde(
        rename = "amount",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub amount: String,
    #[serde(
        rename = "userCurrency",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub user_currency: String,
    #[serde(
        rename = "productInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub product_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "merchantInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub merchant_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "userInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub user_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "goodsInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub goods_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "addressInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub address_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "paymentInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub payment_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "requestedAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub requested_at: String,
    #[serde(
        rename = "updatedAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub updated_at: String,
    #[serde(
        rename = "failedReason",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "FailureReason::is_empty"
    )]
    pub failed_reason: FailureReason,
    #[serde(
        rename = "extendInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub extend_info: String,
    #[serde(
        rename = "subscriptionManagementUrl",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub subscription_management_url: String,
    #[serde(
        rename = "paymentDetails",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub payment_details: Vec<HashMap<String, serde_json::Value>>,

    /// Forward-compat catch-all for server fields the SDK does not yet model.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Result data of a `SUBSCRIPTION_CHANGE_NOTIFICATION` webhook (upgrade /
/// downgrade reaching a final status).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionChangeNotificationResult {
    #[serde(
        rename = "subscriptionRequest",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub subscription_request: String,
    #[serde(
        rename = "originSubscriptionRequest",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub origin_subscription_request: String,
    #[serde(
        rename = "merchantSubscriptionId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub merchant_subscription_id: String,
    #[serde(
        rename = "subscriptionId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub subscription_id: String,
    #[serde(
        rename = "subscriptionChangeStatus",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub subscription_change_status: String,
    #[serde(
        rename = "subscriptionAction",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub subscription_action: String,
    #[serde(
        rename = "remainingAmount",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub remaining_amount: String,
    #[serde(
        rename = "currency",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub currency: String,
    #[serde(
        rename = "userCurrency",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub user_currency: String,
    #[serde(
        rename = "requestedAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub requested_at: String,
    #[serde(
        rename = "subscriptionManagementUrl",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub subscription_management_url: String,
    #[serde(
        rename = "extendInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub extend_info: String,
    #[serde(
        rename = "orderExpiredAt",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub order_expired_at: String,
    #[serde(
        rename = "productInfoList",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub product_info_list: Vec<HashMap<String, serde_json::Value>>,
    #[serde(
        rename = "merchantInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub merchant_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "userInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub user_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "goodsInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub goods_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "addressInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub address_info: HashMap<String, serde_json::Value>,
    #[serde(
        rename = "paymentInfo",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub payment_info: HashMap<String, serde_json::Value>,

    /// Forward-compat catch-all for server fields the SDK does not yet model.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
