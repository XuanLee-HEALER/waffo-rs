//! Order resource (`/order/*`): types, request parameters and the five
//! endpoint free functions (`create`, `inquiry`, `cancel`, `refund`,
//! `capture`).
//!
//! Field names and JSON tags mirror the Go SDK (`types/order/order.go`)
//! byte-for-byte; wire fidelity is required for the signature to match.

// ---------------------------------------------------------------------------
// Nested types (shared by request params and response data)
// ---------------------------------------------------------------------------

/// Merchant information.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MerchantInfo {
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    #[serde(rename = "subMerchantId", skip_serializing_if = "Option::is_none")]
    pub sub_merchant_id: Option<String>,
}

impl waffo_rs::base::MerchantInfoExt for MerchantInfo {
    fn set_merchant_id_if_empty(&mut self, id: &str) {
        if self.merchant_id.as_deref().unwrap_or("").is_empty() {
            self.merchant_id = Some(id.to_string());
        }
    }
}

/// User information.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UserInfo {
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
    #[serde(rename = "userReceiptUrl", skip_serializing_if = "Option::is_none")]
    pub user_receipt_url: Option<String>,
}

/// Payment information.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PaymentInfo {
    #[serde(rename = "productName", skip_serializing_if = "Option::is_none")]
    pub product_name: Option<String>,
    #[serde(rename = "payMethodType", skip_serializing_if = "Option::is_none")]
    pub pay_method_type: Option<String>,
    #[serde(rename = "payMethodName", skip_serializing_if = "Option::is_none")]
    pub pay_method_name: Option<String>,
    #[serde(rename = "payMethodCountry", skip_serializing_if = "Option::is_none")]
    pub pay_method_country: Option<String>,
    #[serde(
        rename = "payMethodUserAccountType",
        skip_serializing_if = "Option::is_none"
    )]
    pub pay_method_user_account_type: Option<String>,
    #[serde(
        rename = "payMethodUserAccountNo",
        skip_serializing_if = "Option::is_none"
    )]
    pub pay_method_user_account_no: Option<String>,
    #[serde(rename = "cashierLanguage", skip_serializing_if = "Option::is_none")]
    pub cashier_language: Option<String>,
    #[serde(
        rename = "userPaymentAccessToken",
        skip_serializing_if = "Option::is_none"
    )]
    pub user_payment_access_token: Option<String>,
    #[serde(rename = "captureMode", skip_serializing_if = "Option::is_none")]
    pub capture_mode: Option<String>,
    #[serde(
        rename = "merchantInitiatedMode",
        skip_serializing_if = "Option::is_none"
    )]
    pub merchant_initiated_mode: Option<String>,
}

/// Goods information.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GoodsInfo {
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

/// Address information (shipping / billing).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AddressInfo {
    #[serde(rename = "shippingAddress", skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<Address>,
    #[serde(rename = "billingAddress", skip_serializing_if = "Option::is_none")]
    pub billing_address: Option<Address>,
}

/// A physical address.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Address {
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

/// Cashier brand display information.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BrandInfo {
    #[serde(rename = "cashierLogoUrl", skip_serializing_if = "Option::is_none")]
    pub cashier_logo_url: Option<String>,
    #[serde(rename = "cashierDisplayName", skip_serializing_if = "Option::is_none")]
    pub cashier_display_name: Option<String>,
    #[serde(
        rename = "cashierProductImageUrl",
        skip_serializing_if = "Option::is_none"
    )]
    pub cashier_product_image_url: Option<String>,
}

/// Card information for direct card payments.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CardInfo {
    #[serde(rename = "cardNumber", skip_serializing_if = "Option::is_none")]
    pub card_number: Option<String>,
    #[serde(rename = "cardExpiryYear", skip_serializing_if = "Option::is_none")]
    pub card_expiry_year: Option<i64>,
    #[serde(rename = "cardExpiryMonth", skip_serializing_if = "Option::is_none")]
    pub card_expiry_month: Option<i64>,
    #[serde(rename = "cardCvv", skip_serializing_if = "Option::is_none")]
    pub card_cvv: Option<String>,
    #[serde(rename = "cardHolderName", skip_serializing_if = "Option::is_none")]
    pub card_holder_name: Option<String>,
    #[serde(rename = "threeDsDecision", skip_serializing_if = "Option::is_none")]
    pub three_ds_decision: Option<String>,
}

/// Tokenized payment data.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PaymentTokenData {
    #[serde(rename = "tokenId", skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
}

/// Product details attached to an order.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProductDetail {
    #[serde(rename = "subscriptionInfo", skip_serializing_if = "Option::is_none")]
    pub subscription_info: Option<crate::biz::subscription::SubscriptionInfo>,
}

/// Risk control auxiliary data.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RiskData {
    #[serde(rename = "userType", skip_serializing_if = "Option::is_none")]
    pub user_type: Option<String>,
    #[serde(rename = "userCategory", skip_serializing_if = "Option::is_none")]
    pub user_category: Option<String>,
    #[serde(rename = "userLegalName", skip_serializing_if = "Option::is_none")]
    pub user_legal_name: Option<String>,
    #[serde(rename = "userDisplayName", skip_serializing_if = "Option::is_none")]
    pub user_display_name: Option<String>,
    #[serde(rename = "userRegistrationIp", skip_serializing_if = "Option::is_none")]
    pub user_registration_ip: Option<String>,
    #[serde(rename = "userLastSeenIp", skip_serializing_if = "Option::is_none")]
    pub user_last_seen_ip: Option<String>,
    #[serde(rename = "userIsNew", skip_serializing_if = "Option::is_none")]
    pub user_is_new: Option<String>,
    #[serde(
        rename = "userIsFirstPurchase",
        skip_serializing_if = "Option::is_none"
    )]
    pub user_is_first_purchase: Option<String>,
}

/// Subscription extension information attached to an acquiring order.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AcqOrderExtSubscriptionInfo {
    #[serde(rename = "subscriptionId", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(rename = "periodNo", skip_serializing_if = "Option::is_none")]
    pub period_no: Option<i64>,
    #[serde(rename = "merchantRequest", skip_serializing_if = "Option::is_none")]
    pub merchant_request: Option<String>,
    #[serde(rename = "subscriptionEvent", skip_serializing_if = "Option::is_none")]
    pub subscription_event: Option<String>,
}

/// Tokenized card data used for direct card payment flows.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WaffoTokenCardData {
    #[serde(
        rename = "cardBinDataList",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub card_bin_data_list: Vec<CardBinData>,
    #[serde(rename = "maskedCardInfo", skip_serializing_if = "Option::is_none")]
    pub masked_card_info: Option<String>,
    #[serde(rename = "cardBin", skip_serializing_if = "Option::is_none")]
    pub card_bin: Option<String>,
    #[serde(rename = "cardExpiry", skip_serializing_if = "Option::is_none")]
    pub card_expiry: Option<String>,
}

/// Card BIN metadata returned with tokenized card data.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CardBinData {
    #[serde(rename = "cardBin", skip_serializing_if = "Option::is_none")]
    pub card_bin: Option<String>,
    #[serde(rename = "cardScheme", skip_serializing_if = "Option::is_none")]
    pub card_scheme: Option<String>,
    #[serde(rename = "cardBrand", skip_serializing_if = "Option::is_none")]
    pub card_brand: Option<String>,
    #[serde(rename = "cardType", skip_serializing_if = "Option::is_none")]
    pub card_type: Option<String>,
    #[serde(
        rename = "cardTypeList",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub card_type_list: Vec<String>,
    #[serde(rename = "subCardType", skip_serializing_if = "Option::is_none")]
    pub sub_card_type: Option<String>,
    #[serde(rename = "cardIssuerName", skip_serializing_if = "Option::is_none")]
    pub card_issuer_name: Option<String>,
    #[serde(rename = "cardIssuerCode", skip_serializing_if = "Option::is_none")]
    pub card_issuer_code: Option<String>,
    #[serde(
        rename = "cardIssueCountryCode",
        skip_serializing_if = "Option::is_none"
    )]
    pub card_issue_country_code: Option<String>,
    #[serde(
        rename = "cardIssueCountryCodeList",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub card_issue_country_code_list: Vec<String>,
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(rename = "extendedInfo", skip_serializing_if = "Option::is_none")]
    pub extended_info: Option<String>,
}

/// User information required for refunds with specific payment methods.
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
    #[serde(rename = "userBankInfo", skip_serializing_if = "Option::is_none")]
    pub user_bank_info: Option<RefundUserBankInfo>,
}

/// Bank account information for a refund user.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RefundUserBankInfo {
    #[serde(rename = "bankAccountNo", skip_serializing_if = "Option::is_none")]
    pub bank_account_no: Option<String>,
    #[serde(rename = "bankCode", skip_serializing_if = "Option::is_none")]
    pub bank_code: Option<String>,
    #[serde(rename = "bankName", skip_serializing_if = "Option::is_none")]
    pub bank_name: Option<String>,
    #[serde(rename = "bankCity", skip_serializing_if = "Option::is_none")]
    pub bank_city: Option<String>,
    #[serde(rename = "bankBranch", skip_serializing_if = "Option::is_none")]
    pub bank_branch: Option<String>,
}

/// The action required for order processing (embedded as a JSON string in the
/// `orderAction` field of responses; parsed by [`CreateOrderData::fetch_redirect_url`]).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OrderAction {
    #[serde(rename = "actionType", skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,
    #[serde(rename = "webUrl", skip_serializing_if = "Option::is_none")]
    pub web_url: Option<String>,
    #[serde(rename = "deeplinkUrl", skip_serializing_if = "Option::is_none")]
    pub deeplink_url: Option<String>,
    #[serde(rename = "actionData", skip_serializing_if = "Option::is_none")]
    pub action_data: Option<OrderActionData>,
}

/// Additional data for an order action (currently empty; mirrors the Go type).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OrderActionData {}

// ---------------------------------------------------------------------------
// create  ->  /order/create
// ---------------------------------------------------------------------------

/// Parameters for creating an order.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct CreateOrderParams {
    #[serde(rename = "paymentRequestId")]
    pub payment_request_id: String,
    #[serde(rename = "merchantOrderId")]
    pub merchant_order_id: String,
    #[serde(rename = "orderCurrency")]
    pub order_currency: String,
    #[serde(rename = "orderAmount")]
    pub order_amount: String,
    #[serde(rename = "userCurrency", skip_serializing_if = "Option::is_none")]
    pub user_currency: Option<String>,
    #[serde(rename = "orderDescription")]
    pub order_description: String,
    #[waffo(requested_at)]
    #[serde(rename = "orderRequestedAt", skip_serializing_if = "Option::is_none")]
    pub order_requested_at: Option<String>,
    #[serde(rename = "orderExpiredAt", skip_serializing_if = "Option::is_none")]
    pub order_expired_at: Option<String>,
    #[serde(rename = "successRedirectUrl", skip_serializing_if = "Option::is_none")]
    pub success_redirect_url: Option<String>,
    #[serde(rename = "failedRedirectUrl", skip_serializing_if = "Option::is_none")]
    pub failed_redirect_url: Option<String>,
    #[serde(rename = "cancelRedirectUrl", skip_serializing_if = "Option::is_none")]
    pub cancel_redirect_url: Option<String>,
    #[serde(rename = "notifyUrl")]
    pub notify_url: String,
    #[serde(rename = "extendInfo", skip_serializing_if = "Option::is_none")]
    pub extend_info: Option<String>,
    #[waffo(merchant_info)]
    #[serde(rename = "merchantInfo", skip_serializing_if = "Option::is_none")]
    pub merchant_info: Option<MerchantInfo>,
    #[serde(rename = "userInfo")]
    pub user_info: Option<UserInfo>,
    #[serde(rename = "goodsInfo", skip_serializing_if = "Option::is_none")]
    pub goods_info: Option<GoodsInfo>,
    #[serde(rename = "paymentInfo")]
    pub payment_info: Option<PaymentInfo>,
    #[serde(rename = "brandInfo", skip_serializing_if = "Option::is_none")]
    pub brand_info: Option<BrandInfo>,
    #[serde(rename = "cardInfo", skip_serializing_if = "Option::is_none")]
    pub card_info: Option<CardInfo>,
    #[serde(rename = "paymentTokenData", skip_serializing_if = "Option::is_none")]
    pub payment_token_data: Option<PaymentTokenData>,
    #[serde(rename = "riskData", skip_serializing_if = "Option::is_none")]
    pub risk_data: Option<RiskData>,
    #[serde(rename = "addressInfo", skip_serializing_if = "Option::is_none")]
    pub address_info: Option<AddressInfo>,
    #[serde(rename = "productDetail", skip_serializing_if = "Option::is_none")]
    pub product_detail: Option<ProductDetail>,
    #[serde(
        rename = "acqOrderExtSubscriptionInfo",
        skip_serializing_if = "Option::is_none"
    )]
    pub acq_order_ext_subscription_info: Option<serde_json::Value>,
    #[serde(rename = "channelId", skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(rename = "innerCardData", skip_serializing_if = "Option::is_none")]
    pub inner_card_data: Option<serde_json::Value>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// Response data for order creation.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CreateOrderData {
    #[serde(rename = "paymentRequestId", skip_serializing_if = "Option::is_none")]
    pub payment_request_id: Option<String>,
    #[serde(rename = "merchantOrderId", skip_serializing_if = "Option::is_none")]
    pub merchant_order_id: Option<String>,
    #[serde(rename = "acquiringOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_order_id: Option<String>,
    #[serde(rename = "orderStatus", skip_serializing_if = "Option::is_none")]
    pub order_status: Option<String>,
    #[serde(rename = "orderAction", skip_serializing_if = "Option::is_none")]
    pub order_action: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl CreateOrderData {
    /// Returns the redirect URL from the order action.
    ///
    /// Parses the `orderAction` JSON string and returns the deeplink URL when
    /// the action type is `DEEPLINK` and a deeplink URL is present, otherwise
    /// the web URL. Returns `""` if `orderAction` is empty or fails to parse.
    pub fn fetch_redirect_url(&self) -> String {
        let raw = match self.order_action.as_deref() {
            Some(s) if !s.is_empty() => s,
            _ => return String::new(),
        };

        let action: OrderAction = match serde_json::from_str(raw) {
            Ok(a) => a,
            Err(_) => return String::new(),
        };

        let deeplink = action.deeplink_url.as_deref().unwrap_or("");
        if action.action_type.as_deref() == Some("DEEPLINK") && !deeplink.is_empty() {
            return deeplink.to_string();
        }

        action.web_url.unwrap_or_default()
    }
}

struct OrderCreate;
impl waffo_rs::base::Endpoint for OrderCreate {
    type Req = CreateOrderParams;
    type Resp = CreateOrderData;
    const PATH: &'static str = "/order/create";
    const READ: bool = false;
}

/// Create a new payment order.
pub async fn create(
    client: &waffo_rs::Client,
    params: CreateOrderParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<CreateOrderData> {
    waffo_rs::base::send_with::<OrderCreate>(client, params, opts).await
}

// ---------------------------------------------------------------------------
// inquiry  ->  /order/inquiry
// ---------------------------------------------------------------------------

/// Parameters for querying an order. Provide `paymentRequestId` or
/// `acquiringOrderId`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct InquiryOrderParams {
    #[serde(rename = "paymentRequestId", skip_serializing_if = "Option::is_none")]
    pub payment_request_id: Option<String>,
    #[serde(rename = "acquiringOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_order_id: Option<String>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// Response data for order inquiry.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InquiryOrderData {
    #[serde(rename = "paymentRequestId", skip_serializing_if = "Option::is_none")]
    pub payment_request_id: Option<String>,
    #[serde(rename = "merchantOrderId", skip_serializing_if = "Option::is_none")]
    pub merchant_order_id: Option<String>,
    #[serde(rename = "acquiringOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_order_id: Option<String>,
    #[serde(rename = "orderStatus", skip_serializing_if = "Option::is_none")]
    pub order_status: Option<String>,
    #[serde(rename = "orderAction", skip_serializing_if = "Option::is_none")]
    pub order_action: Option<String>,
    #[serde(rename = "orderCurrency", skip_serializing_if = "Option::is_none")]
    pub order_currency: Option<String>,
    #[serde(rename = "orderAmount", skip_serializing_if = "Option::is_none")]
    pub order_amount: Option<String>,
    #[serde(rename = "finalDealAmount", skip_serializing_if = "Option::is_none")]
    pub final_deal_amount: Option<String>,
    #[serde(rename = "orderDescription", skip_serializing_if = "Option::is_none")]
    pub order_description: Option<String>,
    #[serde(rename = "merchantInfo", skip_serializing_if = "Option::is_none")]
    pub merchant_info: Option<MerchantInfo>,
    #[serde(rename = "userInfo", skip_serializing_if = "Option::is_none")]
    pub user_info: Option<UserInfo>,
    #[serde(rename = "goodsInfo", skip_serializing_if = "Option::is_none")]
    pub goods_info: Option<GoodsInfo>,
    #[serde(rename = "paymentInfo", skip_serializing_if = "Option::is_none")]
    pub payment_info: Option<PaymentInfo>,
    #[serde(rename = "addressInfo", skip_serializing_if = "Option::is_none")]
    pub address_info: Option<AddressInfo>,
    #[serde(rename = "orderRequestedAt", skip_serializing_if = "Option::is_none")]
    pub order_requested_at: Option<String>,
    #[serde(rename = "orderExpiredAt", skip_serializing_if = "Option::is_none")]
    pub order_expired_at: Option<String>,
    #[serde(rename = "orderUpdatedAt", skip_serializing_if = "Option::is_none")]
    pub order_updated_at: Option<String>,
    #[serde(rename = "orderCompletedAt", skip_serializing_if = "Option::is_none")]
    pub order_completed_at: Option<String>,
    #[serde(rename = "refundExpiryAt", skip_serializing_if = "Option::is_none")]
    pub refund_expiry_at: Option<String>,
    #[serde(rename = "cancelRedirectUrl", skip_serializing_if = "Option::is_none")]
    pub cancel_redirect_url: Option<String>,
    #[serde(rename = "orderFailedReason", skip_serializing_if = "Option::is_none")]
    pub order_failed_reason: Option<String>,
    #[serde(rename = "extendInfo", skip_serializing_if = "Option::is_none")]
    pub extend_info: Option<String>,
    #[serde(rename = "userCurrency", skip_serializing_if = "Option::is_none")]
    pub user_currency: Option<String>,
    #[serde(rename = "subscriptionInfo", skip_serializing_if = "Option::is_none")]
    pub subscription_info: Option<crate::biz::subscription::SubscriptionInfo>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

struct OrderInquiry;
impl waffo_rs::base::Endpoint for OrderInquiry {
    type Req = InquiryOrderParams;
    type Resp = InquiryOrderData;
    const PATH: &'static str = "/order/inquiry";
    const READ: bool = true;
}

/// Query the status of an order.
pub async fn inquiry(
    client: &waffo_rs::Client,
    params: InquiryOrderParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<InquiryOrderData> {
    waffo_rs::base::send_with::<OrderInquiry>(client, params, opts).await
}

// ---------------------------------------------------------------------------
// cancel  ->  /order/cancel
// ---------------------------------------------------------------------------

/// Parameters for canceling an order.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct CancelOrderParams {
    #[serde(rename = "paymentRequestId", skip_serializing_if = "Option::is_none")]
    pub payment_request_id: Option<String>,
    #[serde(rename = "acquiringOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_order_id: Option<String>,
    #[waffo(merchant_id)]
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    #[waffo(requested_at)]
    #[serde(rename = "orderRequestedAt", skip_serializing_if = "Option::is_none")]
    pub order_requested_at: Option<String>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// Response data for order cancellation.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CancelOrderData {
    #[serde(rename = "paymentRequestId", skip_serializing_if = "Option::is_none")]
    pub payment_request_id: Option<String>,
    #[serde(rename = "merchantOrderId", skip_serializing_if = "Option::is_none")]
    pub merchant_order_id: Option<String>,
    #[serde(rename = "acquiringOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_order_id: Option<String>,
    #[serde(rename = "orderStatus", skip_serializing_if = "Option::is_none")]
    pub order_status: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

struct OrderCancel;
impl waffo_rs::base::Endpoint for OrderCancel {
    type Req = CancelOrderParams;
    type Resp = CancelOrderData;
    const PATH: &'static str = "/order/cancel";
    const READ: bool = false;
}

/// Cancel an unpaid order.
pub async fn cancel(
    client: &waffo_rs::Client,
    params: CancelOrderParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<CancelOrderData> {
    waffo_rs::base::send_with::<OrderCancel>(client, params, opts).await
}

// ---------------------------------------------------------------------------
// refund  ->  /order/refund
// ---------------------------------------------------------------------------

/// Parameters for refunding an order.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct RefundOrderParams {
    #[serde(rename = "refundRequestId")]
    pub refund_request_id: String,
    #[serde(rename = "acquiringOrderId")]
    pub acquiring_order_id: String,
    #[serde(
        rename = "merchantRefundOrderId",
        skip_serializing_if = "Option::is_none"
    )]
    pub merchant_refund_order_id: Option<String>,
    #[waffo(merchant_id)]
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    #[waffo(requested_at)]
    #[serde(rename = "requestedAt", skip_serializing_if = "Option::is_none")]
    pub requested_at: Option<String>,
    #[serde(rename = "refundAmount")]
    pub refund_amount: String,
    #[serde(rename = "refundReason")]
    pub refund_reason: String,
    #[serde(rename = "refundNotifyUrl", skip_serializing_if = "Option::is_none")]
    pub notify_url: Option<String>,
    #[serde(rename = "extendInfo", skip_serializing_if = "Option::is_none")]
    pub extend_info: Option<String>,
    #[serde(rename = "refundSource", skip_serializing_if = "Option::is_none")]
    pub refund_source: Option<String>,
    #[serde(rename = "userInfo", skip_serializing_if = "Option::is_none")]
    pub user_info: Option<RefundUserInfo>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// Response data for order refund.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RefundOrderData {
    #[serde(rename = "refundRequestId", skip_serializing_if = "Option::is_none")]
    pub refund_request_id: Option<String>,
    #[serde(
        rename = "merchantRefundOrderId",
        skip_serializing_if = "Option::is_none"
    )]
    pub merchant_refund_order_id: Option<String>,
    #[serde(rename = "acquiringOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_order_id: Option<String>,
    #[serde(
        rename = "acquiringRefundOrderId",
        skip_serializing_if = "Option::is_none"
    )]
    pub acquiring_refund_order_id: Option<String>,
    #[serde(rename = "refundAmount", skip_serializing_if = "Option::is_none")]
    pub refund_amount: Option<String>,
    #[serde(rename = "refundStatus", skip_serializing_if = "Option::is_none")]
    pub refund_status: Option<String>,
    #[serde(
        rename = "remainingRefundAmount",
        skip_serializing_if = "Option::is_none"
    )]
    pub remaining_refund_amount: Option<String>,
    #[serde(rename = "refundSource", skip_serializing_if = "Option::is_none")]
    pub refund_source: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

struct OrderRefund;
impl waffo_rs::base::Endpoint for OrderRefund {
    type Req = RefundOrderParams;
    type Resp = RefundOrderData;
    const PATH: &'static str = "/order/refund";
    const READ: bool = false;
}

/// Request a refund for a paid order.
pub async fn refund(
    client: &waffo_rs::Client,
    params: RefundOrderParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<RefundOrderData> {
    waffo_rs::base::send_with::<OrderRefund>(client, params, opts).await
}

// ---------------------------------------------------------------------------
// capture  ->  /order/capture
// ---------------------------------------------------------------------------

/// Parameters for capturing a pre-authorized payment.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct CaptureOrderParams {
    #[serde(rename = "paymentRequestId", skip_serializing_if = "Option::is_none")]
    pub payment_request_id: Option<String>,
    #[serde(rename = "acquiringOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_order_id: Option<String>,
    #[waffo(merchant_id)]
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    #[waffo(requested_at)]
    #[serde(rename = "captureRequestedAt", skip_serializing_if = "Option::is_none")]
    pub capture_requested_at: Option<String>,
    #[serde(rename = "captureAmount")]
    pub capture_amount: String,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// Response data for order capture.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CaptureOrderData {
    #[serde(rename = "paymentRequestId", skip_serializing_if = "Option::is_none")]
    pub payment_request_id: Option<String>,
    #[serde(rename = "merchantOrderId", skip_serializing_if = "Option::is_none")]
    pub merchant_order_id: Option<String>,
    #[serde(rename = "acquiringOrderId", skip_serializing_if = "Option::is_none")]
    pub acquiring_order_id: Option<String>,
    #[serde(rename = "orderStatus", skip_serializing_if = "Option::is_none")]
    pub order_status: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

struct OrderCapture;
impl waffo_rs::base::Endpoint for OrderCapture {
    type Req = CaptureOrderParams;
    type Resp = CaptureOrderData;
    const PATH: &'static str = "/order/capture";
    const READ: bool = true;
}

/// Capture a pre-authorized payment.
pub async fn capture(
    client: &waffo_rs::Client,
    params: CaptureOrderParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<CaptureOrderData> {
    waffo_rs::base::send_with::<OrderCapture>(client, params, opts).await
}
