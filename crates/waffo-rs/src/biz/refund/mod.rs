//! Refund resource (`/refund/inquiry`).
//!
//! Mirrors `types/refund/refund.go` and `resources/refund_resource.go` from the
//! Go SDK byte-for-byte (field names + JSON tags). Exposes the
//! [`InquiryRefundParams`] / [`InquiryRefundData`] DTOs and the [`inquiry`] free
//! function, which runs through the uniform processing path in [`waffo_rs::base`].

/// Parameters for querying a refund (`POST /refund/inquiry`).
///
/// Has no merchant/requested-at fields, so the derived [`waffo_rs::WaffoRequest`]
/// impl performs no injection.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct InquiryRefundParams {
    #[serde(rename = "refundRequestId")]
    pub refund_request_id: String,
    #[serde(rename = "acquiringRefundOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_refund_order_id: Option<String>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// User information embedded in a refund inquiry response.
///
/// The refund package defines its own version of this block (distinct from the
/// order / subscription `*UserInfo` types).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RefundUserInfo {
    #[serde(rename = "userType", skip_serializing_if = "Option::is_none")]
    pub user_type: Option<String>,
    #[serde(rename = "userFirstName", skip_serializing_if = "Option::is_none")]
    pub user_first_name: Option<String>,
    #[serde(rename = "userMiddleName", skip_serializing_if = "Option::is_none")]
    pub user_middle_name: Option<String>,
    #[serde(rename = "userLastName", skip_serializing_if = "Option::is_none")]
    pub user_last_name: Option<String>,
    #[serde(rename = "nationality", skip_serializing_if = "Option::is_none")]
    pub nationality: Option<String>,
    #[serde(rename = "userEmail", skip_serializing_if = "Option::is_none")]
    pub user_email: Option<String>,
    #[serde(rename = "userPhone", skip_serializing_if = "Option::is_none")]
    pub user_phone: Option<String>,
    #[serde(rename = "userBirthDay", skip_serializing_if = "Option::is_none")]
    pub user_birth_day: Option<String>,
    #[serde(rename = "userIDType", skip_serializing_if = "Option::is_none")]
    pub user_id_type: Option<String>,
    #[serde(rename = "userIDNumber", skip_serializing_if = "Option::is_none")]
    pub user_id_number: Option<String>,
    #[serde(rename = "userIDIssueDate", skip_serializing_if = "Option::is_none")]
    pub user_id_issue_date: Option<String>,
    #[serde(rename = "userIDExpiryDate", skip_serializing_if = "Option::is_none")]
    pub user_id_expiry_date: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Response data for a refund inquiry (`POST /refund/inquiry`).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InquiryRefundData {
    #[serde(rename = "refundRequestId", skip_serializing_if = "Option::is_none")]
    pub refund_request_id: Option<String>,
    #[serde(rename = "merchantRefundOrderId", skip_serializing_if = "Option::is_none")]
    pub merchant_refund_order_id: Option<String>,
    #[serde(rename = "acquiringOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_order_id: Option<String>,
    #[serde(rename = "acquiringRefundOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_refund_order_id: Option<String>,
    #[serde(rename = "origPaymentRequestId", skip_serializing_if = "Option::is_none")]
    pub orig_payment_request_id: Option<String>,
    #[serde(rename = "refundAmount", skip_serializing_if = "Option::is_none")]
    pub refund_amount: Option<String>,
    #[serde(rename = "refundStatus", skip_serializing_if = "Option::is_none")]
    pub refund_status: Option<String>,
    #[serde(rename = "refundReason", skip_serializing_if = "Option::is_none")]
    pub refund_reason: Option<String>,
    #[serde(rename = "refundRequestedAt", skip_serializing_if = "Option::is_none")]
    pub refund_requested_at: Option<String>,
    #[serde(rename = "refundUpdatedAt", skip_serializing_if = "Option::is_none")]
    pub refund_updated_at: Option<String>,
    #[serde(rename = "refundFailedReason", skip_serializing_if = "Option::is_none")]
    pub refund_failed_reason: Option<String>,
    #[serde(rename = "extendInfo", skip_serializing_if = "Option::is_none")]
    pub extend_info: Option<String>,
    #[serde(rename = "userCurrency", skip_serializing_if = "Option::is_none")]
    pub user_currency: Option<String>,
    #[serde(rename = "finalDealAmount", skip_serializing_if = "Option::is_none")]
    pub final_deal_amount: Option<String>,
    #[serde(rename = "merchantUserId", skip_serializing_if = "Option::is_none")]
    pub merchant_user_id: Option<String>,
    #[serde(rename = "subscriptionInfo", skip_serializing_if = "Option::is_none")]
    pub subscription_info: Option<crate::biz::subscription::SubscriptionInfo>,
    #[serde(rename = "remainingRefundAmount", skip_serializing_if = "Option::is_none")]
    pub remaining_refund_amount: Option<String>,
    #[serde(rename = "userInfo", skip_serializing_if = "Option::is_none")]
    pub user_info: Option<RefundUserInfo>,
    #[serde(rename = "refundCompletedAt", skip_serializing_if = "Option::is_none")]
    pub refund_completed_at: Option<String>,
    #[serde(rename = "refundSource", skip_serializing_if = "Option::is_none")]
    pub refund_source: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Endpoint marker for `POST /refund/inquiry` (read / idempotent).
struct InquiryRefund;

impl waffo_rs::base::Endpoint for InquiryRefund {
    type Req = InquiryRefundParams;
    type Resp = InquiryRefundData;
    const PATH: &'static str = "/refund/inquiry";
    const READ: bool = true;
}

/// Query the status of a refund.
pub async fn inquiry(
    client: &waffo_rs::Client,
    params: InquiryRefundParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<InquiryRefundData> {
    waffo_rs::base::send_with::<InquiryRefund>(client, params, opts).await
}
