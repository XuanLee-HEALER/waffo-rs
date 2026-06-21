//! Chargeback resource (`/chargeback/inquiry`, `/chargeback/update`,
//! `/chargeback/accept`, `/chargeback/list`).
//!
//! Implements the Waffo *Chargeback API* (v1, 2025-12) — a business domain not
//! present in the original Go SDK. All four operations reuse the same `{code,
//! msg, data}` envelope, signing and uniform [`send`](waffo_rs::base::send)
//! pipeline as the rest of the SDK. The shared [`ChargebackOrder`] object is
//! also the `result` payload of the `CHARGEBACK_NOTIFICATION` webhook (see
//! [`waffo_rs::webhook`]).
//!
//! The file upload/download endpoints (`/chargeback/file/*`) use a different
//! transport (multipart request / binary-stream response) and are intentionally
//! not part of this module yet.

// ---------------------------------------------------------------------------
// chargebackPhase / chargebackStatus wire constants
// ---------------------------------------------------------------------------

/// Initial chargeback created, awaiting merchant action.
pub const CHARGEBACK_PHASE_NEW: &str = "NEW";
/// Evidence returned to the merchant for resubmission.
pub const CHARGEBACK_PHASE_RETURNED: &str = "RETURNED";
/// Evidence submitted and under review.
pub const CHARGEBACK_PHASE_PROCESSING: &str = "PROCESSING";
/// Chargeback case closed with a final result.
pub const CHARGEBACK_PHASE_FINAL: &str = "FINAL";

/// Initial status: the merchant needs to submit evidence.
pub const CHARGEBACK_STATUS_EVIDENCE_REQUIRED: &str = "EVIDENCE_REQUIRED";
/// Evidence submitted; undergoing channel/risk review.
pub const CHARGEBACK_STATUS_UNDER_REVIEW: &str = "UNDER_REVIEW";
/// Merchant gave up defense. Final; deducts amount & fee.
pub const CHARGEBACK_STATUS_ACCEPTED: &str = "ACCEPTED";
/// Canceled by the channel. Final; deducts fee only.
pub const CHARGEBACK_STATUS_CANCELED: &str = "CANCELED";
/// Channel ruled against the merchant. Final; deducts amount & fee.
pub const CHARGEBACK_STATUS_CASE_LOST: &str = "CASE_LOST";
/// Channel ruled for the merchant. Final; deducts fee only.
pub const CHARGEBACK_STATUS_CASE_WON: &str = "CASE_WON";
/// Evidence submission timed out. Final; deducts amount & fee.
pub const CHARGEBACK_STATUS_EXPIRED: &str = "EXPIRED";
/// Offline settlement reached. Final; deducts fee only.
pub const CHARGEBACK_STATUS_SETTLED: &str = "SETTLED";

// ---------------------------------------------------------------------------
// Strongly-typed classifications (mirrors `webhook::events`: infallible
// `From<&str>`, the raw wire string is always preserved via `Other`).
// ---------------------------------------------------------------------------

/// Classified `chargebackPhase`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChargebackPhase {
    New,
    Returned,
    Processing,
    Final,
    /// Any phase string the SDK does not model (forward-compat).
    Other(String),
}

impl From<&str> for ChargebackPhase {
    fn from(s: &str) -> Self {
        match s {
            CHARGEBACK_PHASE_NEW => ChargebackPhase::New,
            CHARGEBACK_PHASE_RETURNED => ChargebackPhase::Returned,
            CHARGEBACK_PHASE_PROCESSING => ChargebackPhase::Processing,
            CHARGEBACK_PHASE_FINAL => ChargebackPhase::Final,
            other => ChargebackPhase::Other(other.to_string()),
        }
    }
}

impl ChargebackPhase {
    /// The canonical wire string for this phase.
    pub fn as_str(&self) -> &str {
        match self {
            ChargebackPhase::New => CHARGEBACK_PHASE_NEW,
            ChargebackPhase::Returned => CHARGEBACK_PHASE_RETURNED,
            ChargebackPhase::Processing => CHARGEBACK_PHASE_PROCESSING,
            ChargebackPhase::Final => CHARGEBACK_PHASE_FINAL,
            ChargebackPhase::Other(s) => s.as_str(),
        }
    }
}

/// Classified `chargebackStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChargebackStatus {
    EvidenceRequired,
    UnderReview,
    Accepted,
    Canceled,
    CaseLost,
    CaseWon,
    Expired,
    Settled,
    /// Any status string the SDK does not model (forward-compat).
    Other(String),
}

impl From<&str> for ChargebackStatus {
    fn from(s: &str) -> Self {
        match s {
            CHARGEBACK_STATUS_EVIDENCE_REQUIRED => ChargebackStatus::EvidenceRequired,
            CHARGEBACK_STATUS_UNDER_REVIEW => ChargebackStatus::UnderReview,
            CHARGEBACK_STATUS_ACCEPTED => ChargebackStatus::Accepted,
            CHARGEBACK_STATUS_CANCELED => ChargebackStatus::Canceled,
            CHARGEBACK_STATUS_CASE_LOST => ChargebackStatus::CaseLost,
            CHARGEBACK_STATUS_CASE_WON => ChargebackStatus::CaseWon,
            CHARGEBACK_STATUS_EXPIRED => ChargebackStatus::Expired,
            CHARGEBACK_STATUS_SETTLED => ChargebackStatus::Settled,
            other => ChargebackStatus::Other(other.to_string()),
        }
    }
}

impl ChargebackStatus {
    /// The canonical wire string for this status.
    pub fn as_str(&self) -> &str {
        match self {
            ChargebackStatus::EvidenceRequired => CHARGEBACK_STATUS_EVIDENCE_REQUIRED,
            ChargebackStatus::UnderReview => CHARGEBACK_STATUS_UNDER_REVIEW,
            ChargebackStatus::Accepted => CHARGEBACK_STATUS_ACCEPTED,
            ChargebackStatus::Canceled => CHARGEBACK_STATUS_CANCELED,
            ChargebackStatus::CaseLost => CHARGEBACK_STATUS_CASE_LOST,
            ChargebackStatus::CaseWon => CHARGEBACK_STATUS_CASE_WON,
            ChargebackStatus::Expired => CHARGEBACK_STATUS_EXPIRED,
            ChargebackStatus::Settled => CHARGEBACK_STATUS_SETTLED,
            ChargebackStatus::Other(s) => s.as_str(),
        }
    }

    /// True for the terminal statuses (the case is closed; no further evidence
    /// can be submitted). `Other` is treated as non-final.
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            ChargebackStatus::Accepted
                | ChargebackStatus::Canceled
                | ChargebackStatus::CaseLost
                | ChargebackStatus::CaseWon
                | ChargebackStatus::Expired
                | ChargebackStatus::Settled
        )
    }
}

// ---------------------------------------------------------------------------
// Shared sub-objects (used in both request params and the response object)
// ---------------------------------------------------------------------------

/// The `message` sub-object: the merchant↔Waffo messaging thread. On a request
/// (`/chargeback/update`) the merchant fills `notes` / `documents`; on a
/// response `message_id` is assigned by Waffo.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChargebackMessage {
    #[serde(
        rename = "messageId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub message_id: String,
    #[serde(
        rename = "notes",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub notes: String,
    /// Attachment file IDs (comma-separated; obtained from `/chargeback/file/upload`).
    #[serde(
        rename = "documents",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub documents: String,
    /// Forward-compat catch-all for server fields the SDK does not yet model.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// The `evidence` sub-object: free-form defense evidence.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChargebackEvidence {
    /// Any additional evidence text.
    #[serde(
        rename = "othersText",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub others_text: String,
    /// Additional evidence file IDs (comma-separated).
    #[serde(
        rename = "othersFile",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub others_file: String,
    /// Forward-compat catch-all for server fields the SDK does not yet model.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// ChargebackOrder — the shared response object (inquiry / update / accept /
// list record / CHARGEBACK_NOTIFICATION result)
// ---------------------------------------------------------------------------

/// A chargeback order. Returned by [`inquiry`] / [`update`] / [`accept`], used
/// as each record of [`ChargebackListData`], and carried as the `result` of a
/// `CHARGEBACK_NOTIFICATION` webhook.
///
/// `chargeback_phase` / `chargeback_status` are kept as the raw wire strings;
/// use [`ChargebackOrder::phase`] / [`ChargebackOrder::status`] for the
/// classified forms.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChargebackOrder {
    #[serde(
        rename = "chargebackId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub chargeback_id: String,
    /// See [`ChargebackPhase`] for the modeled values.
    #[serde(
        rename = "chargebackPhase",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub chargeback_phase: String,
    #[serde(
        rename = "originalOrderId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub original_order_id: String,
    #[serde(
        rename = "merchantId",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub merchant_id: String,
    /// See [`ChargebackStatus`] for the modeled values.
    #[serde(
        rename = "chargebackStatus",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub chargeback_status: String,
    #[serde(
        rename = "amount",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub amount: String,
    #[serde(
        rename = "currency",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub currency: String,
    #[serde(
        rename = "feeAmount",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub fee_amount: String,
    #[serde(
        rename = "feeCurrency",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub fee_currency: String,
    #[serde(
        rename = "reasonCode",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub reason_code: String,
    #[serde(
        rename = "reason",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub reason: String,
    #[serde(
        rename = "description",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub description: String,
    /// Channel chargeback create time (ISO-8601).
    #[serde(
        rename = "chargebackDateTime",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub chargeback_date_time: String,
    /// Evidence-collection expiry time (ISO-8601).
    #[serde(
        rename = "expiryDateTime",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "String::is_empty"
    )]
    pub expiry_date_time: String,
    #[serde(
        rename = "message",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "Option::is_none"
    )]
    pub message: Option<ChargebackMessage>,
    #[serde(
        rename = "evidence",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "Option::is_none"
    )]
    pub evidence: Option<ChargebackEvidence>,
    /// Forward-compat catch-all for server fields the SDK does not yet model.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl ChargebackOrder {
    /// The classified [`ChargebackPhase`].
    pub fn phase(&self) -> ChargebackPhase {
        ChargebackPhase::from(self.chargeback_phase.as_str())
    }

    /// The classified [`ChargebackStatus`].
    pub fn status(&self) -> ChargebackStatus {
        ChargebackStatus::from(self.chargeback_status.as_str())
    }
}

// ---------------------------------------------------------------------------
// inquiry  ->  /chargeback/inquiry  (read)
// ---------------------------------------------------------------------------

/// Parameters for [`inquiry`]. Provide `original_order_id` **or**
/// `chargeback_id` (at least one); `merchant_id` is auto-injected from the
/// client config when left empty.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct InquiryChargebackParams {
    #[waffo(merchant_id)]
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    #[serde(rename = "originalOrderId", skip_serializing_if = "Option::is_none")]
    pub original_order_id: Option<String>,
    #[serde(rename = "chargebackId", skip_serializing_if = "Option::is_none")]
    pub chargeback_id: Option<String>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

struct ChargebackInquiry;
impl waffo_rs::base::Endpoint for ChargebackInquiry {
    type Req = InquiryChargebackParams;
    type Resp = ChargebackOrder;
    const PATH: &'static str = "/chargeback/inquiry";
    const READ: bool = true;
}

/// Query a chargeback order by its original order id or chargeback id.
pub async fn inquiry(
    client: &waffo_rs::Client,
    params: InquiryChargebackParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<ChargebackOrder> {
    waffo_rs::base::send_with::<ChargebackInquiry>(client, params, opts).await
}

// ---------------------------------------------------------------------------
// update  ->  /chargeback/update  (write; submit defense evidence)
// ---------------------------------------------------------------------------

/// Parameters for [`update`]. Only valid while the chargeback status is
/// `EVIDENCE_REQUIRED`. Provide `message` **or** `evidence` (at least one).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct UpdateChargebackParams {
    #[waffo(merchant_id)]
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    #[serde(rename = "chargebackId")]
    pub chargeback_id: String,
    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    pub message: Option<ChargebackMessage>,
    #[serde(rename = "evidence", skip_serializing_if = "Option::is_none")]
    pub evidence: Option<ChargebackEvidence>,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

struct ChargebackUpdate;
impl waffo_rs::base::Endpoint for ChargebackUpdate {
    type Req = UpdateChargebackParams;
    type Resp = ChargebackOrder;
    const PATH: &'static str = "/chargeback/update";
    const READ: bool = false;
}

/// Submit defense evidence for a chargeback (contest it).
///
/// A successful response means the evidence passed basic validation and was
/// accepted by Waffo; Waffo then forwards it to the payment channel for review.
pub async fn update(
    client: &waffo_rs::Client,
    params: UpdateChargebackParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<ChargebackOrder> {
    waffo_rs::base::send_with::<ChargebackUpdate>(client, params, opts).await
}

// ---------------------------------------------------------------------------
// accept  ->  /chargeback/accept  (write; irreversible)
// ---------------------------------------------------------------------------

/// Parameters for [`accept`].
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct AcceptChargebackParams {
    #[waffo(merchant_id)]
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    #[serde(rename = "chargebackId")]
    pub chargeback_id: String,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

struct ChargebackAccept;
impl waffo_rs::base::Endpoint for ChargebackAccept {
    type Req = AcceptChargebackParams;
    type Resp = ChargebackOrder;
    const PATH: &'static str = "/chargeback/accept";
    const READ: bool = false;
}

/// Accept (give up defending) a chargeback.
///
/// **This action is irreversible.** On acceptance Waffo deducts the chargeback
/// amount and the associated fees, and the merchant can no longer submit any
/// evidence to contest the case.
///
/// (The doc's API-list table calls this `/chargeback/close`; the detailed spec
/// and the sequence diagram call it `/chargeback/accept`, which this SDK uses.)
pub async fn accept(
    client: &waffo_rs::Client,
    params: AcceptChargebackParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<ChargebackOrder> {
    waffo_rs::base::send_with::<ChargebackAccept>(client, params, opts).await
}

// ---------------------------------------------------------------------------
// list  ->  /chargeback/list  (read; paged)
// ---------------------------------------------------------------------------

/// Parameters for [`list`]. `start_time` / `end_time` are ISO-8601 with offset
/// (e.g. `2025-07-01T12:01:01+08:00`) and the window must be within 3 months.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, waffo_rs::WaffoRequest)]
pub struct ListChargebackParams {
    #[waffo(merchant_id)]
    #[serde(rename = "merchantId", skip_serializing_if = "Option::is_none")]
    pub merchant_id: Option<String>,
    /// Optional `chargebackStatus` filter (wire: a JSON array of status strings).
    #[serde(rename = "chargebackStatus", skip_serializing_if = "Option::is_none")]
    pub chargeback_status: Option<Vec<String>>,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    #[serde(rename = "pageNum")]
    pub page_num: i64,
    #[serde(rename = "pageSize")]
    pub page_size: i64,
    #[serde(rename = "extraParams", skip_serializing_if = "Option::is_none")]
    pub extra_params: Option<waffo_rs::ExtraParams>,
}

/// A page of chargeback orders returned by [`list`].
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChargebackListData {
    #[serde(
        rename = "total",
        default,
        deserialize_with = "crate::common::de::null_as_default"
    )]
    pub total: i64,
    #[serde(
        rename = "size",
        default,
        deserialize_with = "crate::common::de::null_as_default"
    )]
    pub size: i64,
    #[serde(
        rename = "current",
        default,
        deserialize_with = "crate::common::de::null_as_default"
    )]
    pub current: i64,
    #[serde(
        rename = "pages",
        default,
        deserialize_with = "crate::common::de::null_as_default"
    )]
    pub pages: i64,
    #[serde(
        rename = "records",
        default,
        deserialize_with = "crate::common::de::null_as_default",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub records: Vec<ChargebackOrder>,
    /// Forward-compat catch-all for server fields the SDK does not yet model.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

struct ChargebackList;
impl waffo_rs::base::Endpoint for ChargebackList {
    type Req = ListChargebackParams;
    type Resp = ChargebackListData;
    const PATH: &'static str = "/chargeback/list";
    const READ: bool = true;
}

/// List chargeback orders in a time window (paged).
pub async fn list(
    client: &waffo_rs::Client,
    params: ListChargebackParams,
    opts: Option<&waffo_rs::RequestOptions>,
) -> waffo_rs::Result<ChargebackListData> {
    waffo_rs::base::send_with::<ChargebackList>(client, params, opts).await
}
