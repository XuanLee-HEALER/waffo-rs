//! Subscription resource (`/subscription/*`): domain models (types +
//! parameters) and endpoint functions.
//!
//! Ported byte-for-byte (field names + JSON tags) from the Go SDK
//! `types/subscription/subscription.go` and `resources/subscription_resource.go`.
//!
//! Operations: `create`, `inquiry`, `cancel`, `manage`, `change`,
//! `change_inquiry`, `update`.

use serde::{Deserialize, Serialize};

use waffo_rs::base::{Endpoint, MerchantInfoExt};

// ===========================================================================
// Shared nested blocks
// ===========================================================================

/// `SubscriptionInfo` represents subscription information in payment order
/// response and webhook notifications. Referenced cross-domain by order/refund.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    #[serde(rename = "subscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub subscription_request: Option<String>,
    #[serde(rename = "merchantRequest", skip_serializing_if = "Option::is_none")]
    pub merchant_request: Option<String>,
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(rename = "period", skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
}

/// `ScheduledAmount` represents a scheduled amount for a specific subscription period.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduledAmount {
    #[serde(rename = "period", skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(rename = "amount", skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

/// `ProductInfo` represents product/billing configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductInfo {
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "periodType")]
    pub period_type: String,
    #[serde(rename = "periodInterval")]
    pub period_interval: String,
    #[serde(rename = "numberOfPeriod", skip_serializing_if = "Option::is_none")]
    pub number_of_period: Option<String>,
    #[serde(rename = "trialPeriodAmount", skip_serializing_if = "Option::is_none")]
    pub trial_period_amount: Option<String>,
    #[serde(rename = "numberOfTrialPeriod", skip_serializing_if = "Option::is_none")]
    pub number_of_trial_period: Option<String>,
    #[serde(rename = "trialPeriodType", skip_serializing_if = "Option::is_none")]
    pub trial_period_type: Option<String>,
    #[serde(rename = "trialPeriodInterval", skip_serializing_if = "Option::is_none")]
    pub trial_period_interval: Option<String>,
    #[serde(rename = "startDateTime", skip_serializing_if = "Option::is_none")]
    pub start_date_time: Option<String>,
    #[serde(rename = "endDateTime", skip_serializing_if = "Option::is_none")]
    pub end_date_time: Option<String>,
    #[serde(rename = "nextPaymentDateTime", skip_serializing_if = "Option::is_none")]
    pub next_payment_date_time: Option<String>,
    #[serde(rename = "currentPeriod", skip_serializing_if = "Option::is_none")]
    pub current_period: Option<String>,
    #[serde(rename = "scheduledAmounts", default, skip_serializing_if = "Vec::is_empty")]
    pub scheduled_amounts: Vec<ScheduledAmount>,
}

/// `SubscriptionMerchantInfo` represents merchant information for subscriptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionMerchantInfo {
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    #[serde(rename = "subMerchantId", skip_serializing_if = "Option::is_none")]
    pub sub_merchant_id: Option<String>,
}

impl MerchantInfoExt for SubscriptionMerchantInfo {
    fn set_merchant_id_if_empty(&mut self, id: &str) {
        if self.merchant_id.as_deref().unwrap_or("").is_empty() {
            self.merchant_id = Some(id.to_string());
        }
    }
}

/// `SubscriptionUserInfo` represents user information for subscriptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionUserInfo {
    #[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(rename = "userEmail", skip_serializing_if = "Option::is_none")]
    pub user_email: Option<String>,
    #[serde(rename = "userPhone", skip_serializing_if = "Option::is_none")]
    pub user_phone: Option<String>,
    #[serde(rename = "userCountryCode", skip_serializing_if = "Option::is_none")]
    pub user_country_code: Option<String>,
    #[serde(rename = "userTerminal", skip_serializing_if = "Option::is_none")]
    pub user_terminal: Option<String>,
    #[serde(rename = "userFirstName", skip_serializing_if = "Option::is_none")]
    pub user_first_name: Option<String>,
    #[serde(rename = "userLastName", skip_serializing_if = "Option::is_none")]
    pub user_last_name: Option<String>,
    #[serde(rename = "userCreatedAt", skip_serializing_if = "Option::is_none")]
    pub user_created_at: Option<String>,
    #[serde(rename = "userBrowserIp", skip_serializing_if = "Option::is_none")]
    pub user_browser_ip: Option<String>,
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

/// `SubscriptionGoodsInfo` represents goods information for subscriptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionGoodsInfo {
    #[serde(rename = "goodsId", skip_serializing_if = "Option::is_none")]
    pub goods_id: Option<String>,
    #[serde(rename = "goodsName", skip_serializing_if = "Option::is_none")]
    pub goods_name: Option<String>,
    #[serde(rename = "goodsCategory", skip_serializing_if = "Option::is_none")]
    pub goods_category: Option<String>,
    #[serde(rename = "goodsUrl", skip_serializing_if = "Option::is_none")]
    pub goods_url: Option<String>,
    #[serde(rename = "appName", skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(rename = "skuName", skip_serializing_if = "Option::is_none")]
    pub sku_name: Option<String>,
    #[serde(rename = "goodsUniquePrice", skip_serializing_if = "Option::is_none")]
    pub goods_unique_price: Option<String>,
    #[serde(rename = "goodsQuantity", skip_serializing_if = "Option::is_none")]
    pub goods_quantity: Option<i64>,
}

/// `SubscriptionAddressInfo` represents address information for subscriptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionAddressInfo {
    #[serde(rename = "shippingAddress", skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<SubscriptionAddress>,
    #[serde(rename = "billingAddress", skip_serializing_if = "Option::is_none")]
    pub billing_address: Option<SubscriptionAddress>,
}

/// `SubscriptionAddress` represents a physical address.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionAddress {
    #[serde(rename = "country", skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(rename = "state", skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(rename = "city", skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(rename = "address1", skip_serializing_if = "Option::is_none")]
    pub address1: Option<String>,
    #[serde(rename = "address2", skip_serializing_if = "Option::is_none")]
    pub address2: Option<String>,
    #[serde(rename = "postalCode", skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
}

/// `SubscriptionBrandInfo` represents cashier brand display information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionBrandInfo {
    #[serde(rename = "cashierLogoUrl", skip_serializing_if = "Option::is_none")]
    pub cashier_logo_url: Option<String>,
    #[serde(rename = "cashierDisplayName", skip_serializing_if = "Option::is_none")]
    pub cashier_display_name: Option<String>,
    #[serde(rename = "cashierProductImageUrl", skip_serializing_if = "Option::is_none")]
    pub cashier_product_image_url: Option<String>,
}

/// `SubscriptionPaymentInfo` represents payment information for subscriptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionPaymentInfo {
    #[serde(rename = "productName", skip_serializing_if = "Option::is_none")]
    pub product_name: Option<String>,
    #[serde(rename = "payMethodType", skip_serializing_if = "Option::is_none")]
    pub pay_method_type: Option<String>,
    #[serde(rename = "payMethodName", skip_serializing_if = "Option::is_none")]
    pub pay_method_name: Option<String>,
    #[serde(rename = "payMethodProperties", skip_serializing_if = "Option::is_none")]
    pub pay_method_properties: Option<String>,
    #[serde(rename = "payMethodPublicUid", skip_serializing_if = "Option::is_none")]
    pub pay_method_public_uid: Option<String>,
    #[serde(rename = "payMethodUserAccessToken", skip_serializing_if = "Option::is_none")]
    pub pay_method_user_access_token: Option<String>,
    #[serde(rename = "payMethodUserAccountType", skip_serializing_if = "Option::is_none")]
    pub pay_method_user_account_type: Option<String>,
    #[serde(rename = "payMethodUserAccountNo", skip_serializing_if = "Option::is_none")]
    pub pay_method_user_account_no: Option<String>,
    #[serde(rename = "payMethodResponse", skip_serializing_if = "Option::is_none")]
    pub pay_method_response: Option<String>,
    #[serde(rename = "cashierLanguage", skip_serializing_if = "Option::is_none")]
    pub cashier_language: Option<String>,
}

/// `SubscriptionRiskData` represents risk control data for subscriptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionRiskData {
    #[serde(rename = "deviceType", skip_serializing_if = "Option::is_none")]
    pub device_type: Option<String>,
    #[serde(rename = "deviceId", skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(rename = "deviceTokenId", skip_serializing_if = "Option::is_none")]
    pub device_token_id: Option<String>,
}

/// `SubscriptionChangeProductInfo` represents product info for subscription change.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionChangeProductInfo {
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "periodType")]
    pub period_type: String,
    #[serde(rename = "periodInterval")]
    pub period_interval: String,
    #[serde(rename = "amount")]
    pub amount: String,
    #[serde(rename = "numberOfPeriod", skip_serializing_if = "Option::is_none")]
    pub number_of_period: Option<String>,
    #[serde(rename = "trialPeriodAmount", skip_serializing_if = "Option::is_none")]
    pub trial_period_amount: Option<String>,
    #[serde(rename = "numberOfTrialPeriod", skip_serializing_if = "Option::is_none")]
    pub number_of_trial_period: Option<String>,
    #[serde(rename = "trialPeriodType", skip_serializing_if = "Option::is_none")]
    pub trial_period_type: Option<String>,
    #[serde(rename = "trialPeriodInterval", skip_serializing_if = "Option::is_none")]
    pub trial_period_interval: Option<String>,
    #[serde(rename = "scheduledAmounts", default, skip_serializing_if = "Vec::is_empty")]
    pub scheduled_amounts: Vec<ScheduledAmount>,
}

/// `UpdateProductInfo` represents product info supported by subscription update.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateProductInfo {
    #[serde(rename = "trialPeriodAmount", skip_serializing_if = "Option::is_none")]
    pub trial_period_amount: Option<String>,
    #[serde(rename = "scheduledAmounts", default, skip_serializing_if = "Vec::is_empty")]
    pub scheduled_amounts: Vec<ScheduledAmount>,
}

/// `PaymentDetail` represents one subscription payment detail returned by inquiry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaymentDetail {
    #[serde(rename = "acquiringOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_order_id: Option<String>,
    #[serde(rename = "orderCurrency", skip_serializing_if = "Option::is_none")]
    pub order_currency: Option<String>,
    #[serde(rename = "orderAmount", skip_serializing_if = "Option::is_none")]
    pub order_amount: Option<String>,
    #[serde(rename = "orderStatus", skip_serializing_if = "Option::is_none")]
    pub order_status: Option<String>,
    #[serde(rename = "orderUpdatedAt", skip_serializing_if = "Option::is_none")]
    pub order_updated_at: Option<String>,
    #[serde(rename = "period", skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
}

// ===========================================================================
// create  (/subscription/create, READ=false)
// ===========================================================================

/// `CreateSubscriptionParams` represents the parameters for creating a subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize, waffo_rs::WaffoRequest)]
pub struct CreateSubscriptionParams {
    #[serde(rename = "subscriptionRequest")]
    pub subscription_request: String,
    #[serde(rename = "merchantSubscriptionId")]
    pub merchant_subscription_id: String,
    #[serde(rename = "currency")]
    pub currency: String,
    #[serde(rename = "amount")]
    pub amount: String,
    #[serde(rename = "userCurrency", skip_serializing_if = "Option::is_none")]
    pub user_currency: Option<String>,
    #[serde(rename = "productInfo")]
    pub product_info: Option<ProductInfo>,
    #[waffo(merchant_info)]
    #[serde(rename = "merchantInfo", skip_serializing_if = "Option::is_none")]
    pub merchant_info: Option<SubscriptionMerchantInfo>,
    #[serde(rename = "userInfo")]
    pub user_info: Option<SubscriptionUserInfo>,
    #[serde(rename = "goodsInfo", skip_serializing_if = "Option::is_none")]
    pub goods_info: Option<SubscriptionGoodsInfo>,
    #[serde(rename = "addressInfo", skip_serializing_if = "Option::is_none")]
    pub address_info: Option<SubscriptionAddressInfo>,
    #[serde(rename = "brandInfo", skip_serializing_if = "Option::is_none")]
    pub brand_info: Option<SubscriptionBrandInfo>,
    #[serde(rename = "paymentInfo")]
    pub payment_info: Option<SubscriptionPaymentInfo>,
    #[serde(rename = "riskData", skip_serializing_if = "Option::is_none")]
    pub risk_data: Option<SubscriptionRiskData>,
    #[waffo(requested_at)]
    #[serde(rename = "requestedAt", skip_serializing_if = "Option::is_none")]
    pub requested_at: Option<String>,
    #[serde(rename = "orderExpiredAt", skip_serializing_if = "Option::is_none")]
    pub order_expired_at: Option<String>,
    #[serde(rename = "successRedirectUrl", skip_serializing_if = "Option::is_none")]
    pub success_redirect_url: Option<String>,
    #[serde(rename = "failedRedirectUrl", skip_serializing_if = "Option::is_none")]
    pub failed_redirect_url: Option<String>,
    #[serde(rename = "cancelRedirectUrl", skip_serializing_if = "Option::is_none")]
    pub cancel_redirect_url: Option<String>,
    #[serde(rename = "subscriptionManagementUrl", skip_serializing_if = "Option::is_none")]
    pub subscription_management_url: Option<String>,
    #[serde(rename = "notifyUrl")]
    pub notify_url: String,
    #[serde(rename = "extendInfo", skip_serializing_if = "Option::is_none")]
    pub extend_info: Option<String>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    #[serde(rename = "subscriptionMode", skip_serializing_if = "Option::is_none")]
    pub subscription_mode: Option<String>,
    #[serde(rename = "acqMerchant", skip_serializing_if = "Option::is_none")]
    pub acq_merchant: Option<serde_json::Value>,
    #[serde(rename = "acqProduct", skip_serializing_if = "Option::is_none")]
    pub acq_product: Option<serde_json::Value>,
    #[serde(rename = "acqAgreements", default, skip_serializing_if = "Vec::is_empty")]
    pub acq_agreements: Vec<serde_json::Value>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// `CreateSubscriptionData` represents the response data for subscription creation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateSubscriptionData {
    #[serde(rename = "subscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub subscription_request: Option<String>,
    #[serde(rename = "merchantSubscriptionId", skip_serializing_if = "Option::is_none")]
    pub merchant_subscription_id: Option<String>,
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(rename = "payMethodSubscriptionId", skip_serializing_if = "Option::is_none")]
    pub pay_method_subscription_id: Option<String>,
    #[serde(rename = "subscriptionStatus", skip_serializing_if = "Option::is_none")]
    pub subscription_status: Option<String>,
    #[serde(rename = "subscriptionAction", skip_serializing_if = "Option::is_none")]
    pub subscription_action: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl CreateSubscriptionData {
    /// Returns the redirect URL from the subscription action.
    ///
    /// Matches Go `CreateSubscriptionData.FetchRedirectURL`: if empty -> `""`;
    /// trim; if it starts with `http://`/`https://` return it; otherwise parse
    /// as JSON `{webUrl, deeplinkUrl}` and return `webUrl` if non-empty, else
    /// `deeplinkUrl` if non-empty, else `""`.
    pub fn fetch_redirect_url(&self) -> String {
        let action = self.subscription_action.as_deref().unwrap_or("");
        if action.is_empty() {
            return String::new();
        }
        let trimmed = action.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return trimmed.to_string();
        }
        #[derive(Deserialize)]
        struct Action {
            #[serde(rename = "webUrl", default)]
            web_url: String,
            #[serde(rename = "deeplinkUrl", default)]
            deeplink_url: String,
        }
        if let Ok(a) = serde_json::from_str::<Action>(trimmed) {
            if !a.web_url.is_empty() {
                return a.web_url;
            }
            if !a.deeplink_url.is_empty() {
                return a.deeplink_url;
            }
        }
        String::new()
    }
}

struct CreateOp;
impl Endpoint for CreateOp {
    type Req = CreateSubscriptionParams;
    type Resp = CreateSubscriptionData;
    const PATH: &'static str = "/subscription/create";
    const READ: bool = false;
}

/// Create a new subscription (`POST /subscription/create`).
pub async fn create(
    client: &waffo_rs::Client,
    params: CreateSubscriptionParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<CreateSubscriptionData> {
    waffo_rs::base::send_with::<CreateOp>(client, params, opts).await
}

// ===========================================================================
// inquiry  (/subscription/inquiry, READ=true)
// ===========================================================================

/// `InquirySubscriptionParams` represents the parameters for querying a subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize, waffo_rs::WaffoRequest)]
pub struct InquirySubscriptionParams {
    #[serde(rename = "subscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub subscription_request: Option<String>,
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(rename = "paymentDetails", skip_serializing_if = "Option::is_none")]
    pub payment_details: Option<i64>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// `InquirySubscriptionData` represents the response data for subscription inquiry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InquirySubscriptionData {
    #[serde(rename = "subscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub subscription_request: Option<String>,
    #[serde(rename = "merchantSubscriptionId", skip_serializing_if = "Option::is_none")]
    pub merchant_subscription_id: Option<String>,
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(rename = "payMethodSubscriptionId", skip_serializing_if = "Option::is_none")]
    pub pay_method_subscription_id: Option<String>,
    #[serde(rename = "subscriptionStatus", skip_serializing_if = "Option::is_none")]
    pub subscription_status: Option<String>,
    #[serde(rename = "subscriptionAction", skip_serializing_if = "Option::is_none")]
    pub subscription_action: Option<String>,
    #[serde(rename = "currency", skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(rename = "userCurrency", skip_serializing_if = "Option::is_none")]
    pub user_currency: Option<String>,
    #[serde(rename = "amount", skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    #[serde(rename = "productInfo", skip_serializing_if = "Option::is_none")]
    pub product_info: Option<ProductInfo>,
    #[serde(rename = "merchantInfo", skip_serializing_if = "Option::is_none")]
    pub merchant_info: Option<SubscriptionMerchantInfo>,
    #[serde(rename = "userInfo", skip_serializing_if = "Option::is_none")]
    pub user_info: Option<SubscriptionUserInfo>,
    #[serde(rename = "paymentInfo", skip_serializing_if = "Option::is_none")]
    pub payment_info: Option<SubscriptionPaymentInfo>,
    #[serde(rename = "requestedAt", skip_serializing_if = "Option::is_none")]
    pub requested_at: Option<String>,
    #[serde(rename = "updatedAt", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(rename = "failedReason", skip_serializing_if = "Option::is_none")]
    pub failed_reason: Option<String>,
    #[serde(rename = "subscriptionManagementUrl", skip_serializing_if = "Option::is_none")]
    pub subscription_management_url: Option<String>,
    #[serde(rename = "extendInfo", skip_serializing_if = "Option::is_none")]
    pub extend_info: Option<String>,
    #[serde(rename = "paymentDetails", default, skip_serializing_if = "Vec::is_empty")]
    pub payment_details: Vec<PaymentDetail>,
    #[serde(rename = "goodsInfo", skip_serializing_if = "Option::is_none")]
    pub goods_info: Option<SubscriptionGoodsInfo>,
    #[serde(rename = "addressInfo", skip_serializing_if = "Option::is_none")]
    pub address_info: Option<SubscriptionAddressInfo>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

struct InquiryOp;
impl Endpoint for InquiryOp {
    type Req = InquirySubscriptionParams;
    type Resp = InquirySubscriptionData;
    const PATH: &'static str = "/subscription/inquiry";
    const READ: bool = true;
}

/// Query the status of a subscription (`POST /subscription/inquiry`).
pub async fn inquiry(
    client: &waffo_rs::Client,
    params: InquirySubscriptionParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<InquirySubscriptionData> {
    waffo_rs::base::send_with::<InquiryOp>(client, params, opts).await
}

// ===========================================================================
// cancel  (/subscription/cancel, READ=false)
// ===========================================================================

/// `CancelSubscriptionParams` represents the parameters for canceling a subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize, waffo_rs::WaffoRequest)]
pub struct CancelSubscriptionParams {
    #[serde(rename = "subscriptionId")]
    pub subscription_id: String,
    #[waffo(merchant_id)]
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    #[waffo(requested_at)]
    #[serde(rename = "requestedAt", skip_serializing_if = "Option::is_none")]
    pub requested_at: Option<String>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// `CancelSubscriptionData` represents the response data for subscription cancellation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CancelSubscriptionData {
    #[serde(rename = "merchantSubscriptionId", skip_serializing_if = "Option::is_none")]
    pub merchant_subscription_id: Option<String>,
    #[serde(rename = "subscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub subscription_request: Option<String>,
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(rename = "orderStatus", skip_serializing_if = "Option::is_none")]
    pub order_status: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

struct CancelOp;
impl Endpoint for CancelOp {
    type Req = CancelSubscriptionParams;
    type Resp = CancelSubscriptionData;
    const PATH: &'static str = "/subscription/cancel";
    const READ: bool = false;
}

/// Cancel a subscription (`POST /subscription/cancel`).
pub async fn cancel(
    client: &waffo_rs::Client,
    params: CancelSubscriptionParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<CancelSubscriptionData> {
    waffo_rs::base::send_with::<CancelOp>(client, params, opts).await
}

// ===========================================================================
// manage  (/subscription/manage, READ=true)
// ===========================================================================

/// `ManageSubscriptionParams` represents the parameters for managing a subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize, waffo_rs::WaffoRequest)]
pub struct ManageSubscriptionParams {
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(rename = "subscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub subscription_request: Option<String>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// `ManageSubscriptionData` represents the response data for subscription management.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManageSubscriptionData {
    #[serde(rename = "subscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub subscription_request: Option<String>,
    #[serde(rename = "merchantSubscriptionId", skip_serializing_if = "Option::is_none")]
    pub merchant_subscription_id: Option<String>,
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(rename = "managementUrl", skip_serializing_if = "Option::is_none")]
    pub management_url: Option<String>,
    #[serde(rename = "expiredAt", skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<String>,
    #[serde(rename = "subscriptionStatus", skip_serializing_if = "Option::is_none")]
    pub subscription_status: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

struct ManageOp;
impl Endpoint for ManageOp {
    type Req = ManageSubscriptionParams;
    type Resp = ManageSubscriptionData;
    const PATH: &'static str = "/subscription/manage";
    const READ: bool = true;
}

/// Get the subscription management URL (`POST /subscription/manage`).
pub async fn manage(
    client: &waffo_rs::Client,
    params: ManageSubscriptionParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<ManageSubscriptionData> {
    waffo_rs::base::send_with::<ManageOp>(client, params, opts).await
}

// ===========================================================================
// change  (/subscription/change, READ=false)
// ===========================================================================

/// `ChangeSubscriptionParams` represents the parameters for changing a subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize, waffo_rs::WaffoRequest)]
pub struct ChangeSubscriptionParams {
    #[serde(rename = "subscriptionRequest")]
    pub subscription_request: String,
    #[serde(rename = "merchantSubscriptionId", skip_serializing_if = "Option::is_none")]
    pub merchant_subscription_id: Option<String>,
    #[serde(rename = "originSubscriptionRequest")]
    pub origin_subscription_request: String,
    #[serde(rename = "remainingAmount")]
    pub remaining_amount: String,
    #[serde(rename = "currency")]
    pub currency: String,
    #[serde(rename = "userCurrency", skip_serializing_if = "Option::is_none")]
    pub user_currency: Option<String>,
    #[waffo(requested_at)]
    #[serde(rename = "requestedAt")]
    pub requested_at: Option<String>,
    #[serde(rename = "successRedirectUrl", skip_serializing_if = "Option::is_none")]
    pub success_redirect_url: Option<String>,
    #[serde(rename = "failedRedirectUrl", skip_serializing_if = "Option::is_none")]
    pub failed_redirect_url: Option<String>,
    #[serde(rename = "cancelRedirectUrl", skip_serializing_if = "Option::is_none")]
    pub cancel_redirect_url: Option<String>,
    #[serde(rename = "notifyUrl")]
    pub notify_url: String,
    #[serde(rename = "subscriptionManagementUrl", skip_serializing_if = "Option::is_none")]
    pub subscription_management_url: Option<String>,
    #[serde(rename = "extendInfo", skip_serializing_if = "Option::is_none")]
    pub extend_info: Option<String>,
    #[serde(rename = "orderExpiredAt", skip_serializing_if = "Option::is_none")]
    pub order_expired_at: Option<String>,
    #[serde(rename = "productInfoList", default)]
    pub product_info_list: Vec<SubscriptionChangeProductInfo>,
    #[waffo(merchant_info)]
    #[serde(rename = "merchantInfo")]
    pub merchant_info: Option<SubscriptionMerchantInfo>,
    #[serde(rename = "userInfo")]
    pub user_info: Option<SubscriptionUserInfo>,
    #[serde(rename = "goodsInfo")]
    pub goods_info: Option<SubscriptionGoodsInfo>,
    #[serde(rename = "addressInfo", skip_serializing_if = "Option::is_none")]
    pub address_info: Option<SubscriptionAddressInfo>,
    #[serde(rename = "brandInfo", skip_serializing_if = "Option::is_none")]
    pub brand_info: Option<SubscriptionBrandInfo>,
    #[serde(rename = "paymentInfo")]
    pub payment_info: Option<SubscriptionPaymentInfo>,
    #[serde(rename = "riskData", skip_serializing_if = "Option::is_none")]
    pub risk_data: Option<SubscriptionRiskData>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// `ChangeSubscriptionData` represents the response data for subscription change.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChangeSubscriptionData {
    #[serde(rename = "originSubscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub origin_subscription_request: Option<String>,
    #[serde(rename = "subscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub subscription_request: Option<String>,
    #[serde(rename = "merchantSubscriptionId", skip_serializing_if = "Option::is_none")]
    pub merchant_subscription_id: Option<String>,
    #[serde(rename = "subscriptionChangeStatus", skip_serializing_if = "Option::is_none")]
    pub subscription_change_status: Option<String>,
    #[serde(rename = "subscriptionAction", skip_serializing_if = "Option::is_none")]
    pub subscription_action: Option<String>,
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl ChangeSubscriptionData {
    /// Returns the redirect URL from the subscription action.
    ///
    /// Matches Go `ChangeSubscriptionData.FetchRedirectURL`: if empty -> `""`;
    /// trim; if it starts with `http://`/`https://` return it; otherwise parse
    /// as JSON `{webUrl}` and return `webUrl` if non-empty, else `""`.
    pub fn fetch_redirect_url(&self) -> String {
        let action = self.subscription_action.as_deref().unwrap_or("");
        if action.is_empty() {
            return String::new();
        }
        let trimmed = action.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return trimmed.to_string();
        }
        #[derive(Deserialize)]
        struct Action {
            #[serde(rename = "webUrl", default)]
            web_url: String,
        }
        if let Ok(a) = serde_json::from_str::<Action>(trimmed) {
            if !a.web_url.is_empty() {
                return a.web_url;
            }
        }
        String::new()
    }
}

struct ChangeOp;
impl Endpoint for ChangeOp {
    type Req = ChangeSubscriptionParams;
    type Resp = ChangeSubscriptionData;
    const PATH: &'static str = "/subscription/change";
    const READ: bool = false;
}

/// Change (upgrade/downgrade) a subscription (`POST /subscription/change`).
pub async fn change(
    client: &waffo_rs::Client,
    params: ChangeSubscriptionParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<ChangeSubscriptionData> {
    waffo_rs::base::send_with::<ChangeOp>(client, params, opts).await
}

// ===========================================================================
// change_inquiry  (/subscription/change/inquiry, READ=true)
// ===========================================================================

/// `ChangeInquiryParams` represents the parameters for querying a subscription change.
#[derive(Debug, Clone, Default, Serialize, Deserialize, waffo_rs::WaffoRequest)]
pub struct ChangeInquiryParams {
    #[serde(rename = "originSubscriptionRequest")]
    pub origin_subscription_request: String,
    #[serde(rename = "subscriptionRequest")]
    pub subscription_request: String,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// `ChangeInquiryData` represents the response data for subscription change inquiry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChangeInquiryData {
    #[serde(rename = "subscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub subscription_request: Option<String>,
    #[serde(rename = "originSubscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub origin_subscription_request: Option<String>,
    #[serde(rename = "merchantSubscriptionId", skip_serializing_if = "Option::is_none")]
    pub merchant_subscription_id: Option<String>,
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(rename = "subscriptionChangeStatus", skip_serializing_if = "Option::is_none")]
    pub subscription_change_status: Option<String>,
    #[serde(rename = "subscriptionAction", skip_serializing_if = "Option::is_none")]
    pub subscription_action: Option<String>,
    #[serde(rename = "remainingAmount", skip_serializing_if = "Option::is_none")]
    pub remaining_amount: Option<String>,
    #[serde(rename = "currency", skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(rename = "userCurrency", skip_serializing_if = "Option::is_none")]
    pub user_currency: Option<String>,
    #[serde(rename = "requestedAt", skip_serializing_if = "Option::is_none")]
    pub requested_at: Option<String>,
    #[serde(rename = "subscriptionManagementUrl", skip_serializing_if = "Option::is_none")]
    pub subscription_management_url: Option<String>,
    #[serde(rename = "extendInfo", skip_serializing_if = "Option::is_none")]
    pub extend_info: Option<String>,
    #[serde(rename = "orderExpiredAt", skip_serializing_if = "Option::is_none")]
    pub order_expired_at: Option<String>,
    #[serde(rename = "productInfoList", default, skip_serializing_if = "Vec::is_empty")]
    pub product_info_list: Vec<SubscriptionChangeProductInfo>,
    #[serde(rename = "merchantInfo", skip_serializing_if = "Option::is_none")]
    pub merchant_info: Option<SubscriptionMerchantInfo>,
    #[serde(rename = "userInfo", skip_serializing_if = "Option::is_none")]
    pub user_info: Option<SubscriptionUserInfo>,
    #[serde(rename = "goodsInfo", skip_serializing_if = "Option::is_none")]
    pub goods_info: Option<SubscriptionGoodsInfo>,
    #[serde(rename = "addressInfo", skip_serializing_if = "Option::is_none")]
    pub address_info: Option<SubscriptionAddressInfo>,
    #[serde(rename = "paymentInfo", skip_serializing_if = "Option::is_none")]
    pub payment_info: Option<SubscriptionPaymentInfo>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

struct ChangeInquiryOp;
impl Endpoint for ChangeInquiryOp {
    type Req = ChangeInquiryParams;
    type Resp = ChangeInquiryData;
    const PATH: &'static str = "/subscription/change/inquiry";
    const READ: bool = true;
}

/// Query the status of a subscription change (`POST /subscription/change/inquiry`).
pub async fn change_inquiry(
    client: &waffo_rs::Client,
    params: ChangeInquiryParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<ChangeInquiryData> {
    waffo_rs::base::send_with::<ChangeInquiryOp>(client, params, opts).await
}

// ===========================================================================
// update  (/subscription/update, READ=false)
// ===========================================================================

/// `UpdateSubscriptionParams` represents the parameters for updating a subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize, waffo_rs::WaffoRequest)]
pub struct UpdateSubscriptionParams {
    #[serde(rename = "subscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub subscription_request: Option<String>,
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(rename = "amount", skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    #[serde(rename = "productInfo", skip_serializing_if = "Option::is_none")]
    pub product_info: Option<UpdateProductInfo>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// `UpdateSubscriptionData` represents the response data for subscription update.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSubscriptionData {
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(rename = "subscriptionRequest", skip_serializing_if = "Option::is_none")]
    pub subscription_request: Option<String>,
    #[serde(rename = "previousTrialPeriodAmount", skip_serializing_if = "Option::is_none")]
    pub previous_trial_period_amount: Option<String>,
    #[serde(rename = "newTrialPeriodAmount", skip_serializing_if = "Option::is_none")]
    pub new_trial_period_amount: Option<String>,
    #[serde(rename = "previousAmount", skip_serializing_if = "Option::is_none")]
    pub previous_amount: Option<String>,
    #[serde(rename = "newAmount", skip_serializing_if = "Option::is_none")]
    pub new_amount: Option<String>,
    #[serde(rename = "previousScheduledAmounts", default, skip_serializing_if = "Vec::is_empty")]
    pub previous_scheduled_amounts: Vec<ScheduledAmount>,
    #[serde(rename = "newScheduledAmounts", default, skip_serializing_if = "Vec::is_empty")]
    pub new_scheduled_amounts: Vec<ScheduledAmount>,
    #[serde(rename = "nextEffectivePeriod", skip_serializing_if = "Option::is_none")]
    pub next_effective_period: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

struct UpdateOp;
impl Endpoint for UpdateOp {
    type Req = UpdateSubscriptionParams;
    type Resp = UpdateSubscriptionData;
    const PATH: &'static str = "/subscription/update";
    const READ: bool = false;
}

/// Update an existing subscription's amount, trial amount, or scheduled amounts
/// (`POST /subscription/update`).
pub async fn update(
    client: &waffo_rs::Client,
    params: UpdateSubscriptionParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<UpdateSubscriptionData> {
    waffo_rs::base::send_with::<UpdateOp>(client, params, opts).await
}
