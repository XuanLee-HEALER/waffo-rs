//! Merchant config (`/merchantconfig/inquiry`) and pay-method config
//! (`/paymethodconfig/inquiry`).
//!
//! Both operations are read/idempotent inquiries: `merchantId` is a required
//! top-level field (no injection — it is mandatory in the Go params), and a
//! transport failure maps to [`WaffoError::UnknownStatus`] (Go's E0001).
//!
//! [`WaffoError::UnknownStatus`]: waffo_rs::WaffoError::UnknownStatus

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// merchant config
// ---------------------------------------------------------------------------

/// Parameters for querying merchant config (`/merchantconfig/inquiry`).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct InquiryMerchantConfigParams {
    /// Merchant id (required; not auto-injected).
    #[serde(rename = "merchantId")]
    pub merchant_id: String,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// Response data for a merchant config inquiry.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InquiryMerchantConfigData {
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    #[serde(
        rename = "totalDailyLimit",
        default,
        skip_serializing_if = "std::collections::HashMap::is_empty"
    )]
    pub total_daily_limit: HashMap<String, String>,
    #[serde(
        rename = "remainingDailyLimit",
        default,
        skip_serializing_if = "std::collections::HashMap::is_empty"
    )]
    pub remaining_daily_limit: HashMap<String, String>,
    #[serde(
        rename = "transactionLimit",
        default,
        skip_serializing_if = "std::collections::HashMap::is_empty"
    )]
    pub transaction_limit: HashMap<String, String>,
    /// Forward-compat catch-all for server-added fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// pay-method config
// ---------------------------------------------------------------------------

/// Parameters for querying payment method config (`/paymethodconfig/inquiry`).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct InquiryPayMethodConfigParams {
    /// Merchant id (required; not auto-injected).
    #[serde(rename = "merchantId")]
    pub merchant_id: String,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// Response data for a payment method config inquiry.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InquiryPayMethodConfigData {
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    #[serde(
        rename = "payMethodDetails",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub pay_method_details: Vec<PayMethodDetail>,
    /// Forward-compat catch-all for server-added fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A single payment method detail returned by a pay-method config inquiry.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PayMethodDetail {
    #[serde(rename = "productName", skip_serializing_if = "Option::is_none")]
    pub product_name: Option<String>,
    #[serde(rename = "payMethodName", skip_serializing_if = "Option::is_none")]
    pub pay_method_name: Option<String>,
    #[serde(rename = "country", skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(rename = "currentStatus", skip_serializing_if = "Option::is_none")]
    pub current_status: Option<String>,
    #[serde(
        rename = "fixedMaintenanceRules",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub fixed_maintenance_rules: Vec<FixedMaintenanceRule>,
    #[serde(
        rename = "fixedMaintenanceTimezone",
        skip_serializing_if = "Option::is_none"
    )]
    pub fixed_maintenance_timezone: Option<String>,
    /// Forward-compat catch-all for server-added fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A fixed maintenance rule (start/end cron-like rule strings).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FixedMaintenanceRule {
    #[serde(rename = "startRule", skip_serializing_if = "Option::is_none")]
    pub start_rule: Option<String>,
    #[serde(rename = "endRule", skip_serializing_if = "Option::is_none")]
    pub end_rule: Option<String>,
    /// Forward-compat catch-all for server-added fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// endpoints + free functions
// ---------------------------------------------------------------------------

struct MerchantConfigInquiry;

impl waffo_rs::base::Endpoint for MerchantConfigInquiry {
    type Req = InquiryMerchantConfigParams;
    type Resp = InquiryMerchantConfigData;
    const PATH: &'static str = "/merchantconfig/inquiry";
    const READ: bool = true;
}

/// Query the merchant configuration (`/merchantconfig/inquiry`).
pub async fn merchant_config_inquiry(
    client: &waffo_rs::Client,
    params: InquiryMerchantConfigParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<InquiryMerchantConfigData> {
    waffo_rs::base::send_with::<MerchantConfigInquiry>(client, params, opts).await
}

struct PayMethodConfigInquiry;

impl waffo_rs::base::Endpoint for PayMethodConfigInquiry {
    type Req = InquiryPayMethodConfigParams;
    type Resp = InquiryPayMethodConfigData;
    const PATH: &'static str = "/paymethodconfig/inquiry";
    const READ: bool = true;
}

/// Query the payment method configuration (`/paymethodconfig/inquiry`).
pub async fn pay_method_config_inquiry(
    client: &waffo_rs::Client,
    params: InquiryPayMethodConfigParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<InquiryPayMethodConfigData> {
    waffo_rs::base::send_with::<PayMethodConfigInquiry>(client, params, opts).await
}
