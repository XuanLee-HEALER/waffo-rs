//! Webhook event/status constants, the [`WebhookEvent`] enum and the
//! fine-grained classification helpers built on top of it.
//!
//! Constants mirror `core/webhook_handler.go` byte-for-byte.

use super::notification::{
    PaymentNotificationResult, RefundNotificationResult, SubscriptionChangeNotificationResult,
    SubscriptionNotificationResult,
};

// ---------------------------------------------------------------------------
// Event types (the `eventType` envelope field).
// ---------------------------------------------------------------------------

/// `eventType` for a payment notification.
pub const EVENT_PAYMENT: &str = "PAYMENT_NOTIFICATION";
/// `eventType` for a refund notification.
pub const EVENT_REFUND: &str = "REFUND_NOTIFICATION";
/// `eventType` for a subscription status notification.
pub const EVENT_SUBSCRIPTION_STATUS: &str = "SUBSCRIPTION_STATUS_NOTIFICATION";
/// `eventType` for a subscription period-changed notification.
pub const EVENT_SUBSCRIPTION_PERIOD_CHANGED: &str = "SUBSCRIPTION_PERIOD_CHANGED_NOTIFICATION";
/// `eventType` for a subscription change (upgrade/downgrade) notification.
pub const EVENT_SUBSCRIPTION_CHANGE: &str = "SUBSCRIPTION_CHANGE_NOTIFICATION";

// ---------------------------------------------------------------------------
// Order status values (PAYMENT_NOTIFICATION `orderStatus`).
// ---------------------------------------------------------------------------

/// Order accepted, payment in progress.
pub const ORDER_STATUS_PAY_IN_PROGRESS: &str = "PAY_IN_PROGRESS";
/// User authorization required (redirect to the payment page).
pub const ORDER_STATUS_AUTHORIZATION_REQUIRED: &str = "AUTHORIZATION_REQUIRED";
/// Authorized, waiting for the merchant to capture (CARD only).
pub const ORDER_STATUS_AUTHED_WAITING_CAPTURE: &str = "AUTHED_WAITING_CAPTURE";
/// Payment completed.
pub const ORDER_STATUS_PAY_SUCCESS: &str = "PAY_SUCCESS";
/// Order closed (cancelled, failed, or expired).
pub const ORDER_STATUS_ORDER_CLOSE: &str = "ORDER_CLOSE";

// ---------------------------------------------------------------------------
// Refund status values (REFUND_NOTIFICATION `refundStatus`).
// ---------------------------------------------------------------------------

/// Refund is being processed (async).
pub const REFUND_STATUS_IN_PROGRESS: &str = "REFUND_IN_PROGRESS";
/// Order partially refunded.
pub const REFUND_STATUS_PARTIALLY_REFUNDED: &str = "ORDER_PARTIALLY_REFUNDED";
/// Order fully refunded.
pub const REFUND_STATUS_FULLY_REFUNDED: &str = "ORDER_FULLY_REFUNDED";
/// Refund failed.
pub const REFUND_STATUS_FAILED: &str = "ORDER_REFUND_FAILED";

// ---------------------------------------------------------------------------
// Subscription status values (SUBSCRIPTION_STATUS_NOTIFICATION
// `subscriptionStatus`).
// ---------------------------------------------------------------------------

/// User authorization required.
pub const SUBSCRIPTION_STATUS_AUTHORIZATION_REQUIRED: &str = "AUTHORIZATION_REQUIRED";
/// Subscription being processed.
pub const SUBSCRIPTION_STATUS_IN_PROGRESS: &str = "IN_PROGRESS";
/// Subscription active and billing.
pub const SUBSCRIPTION_STATUS_ACTIVE: &str = "ACTIVE";
/// Subscription closed (timeout or failed).
pub const SUBSCRIPTION_STATUS_CLOSE: &str = "CLOSE";
/// Cancelled by merchant.
pub const SUBSCRIPTION_STATUS_MERCHANT_CANCELLED: &str = "MERCHANT_CANCELLED";
/// Cancelled by user.
pub const SUBSCRIPTION_STATUS_USER_CANCELLED: &str = "USER_CANCELLED";
/// Cancelled by channel.
pub const SUBSCRIPTION_STATUS_CHANNEL_CANCELLED: &str = "CHANNEL_CANCELLED";
/// Subscription expired.
pub const SUBSCRIPTION_STATUS_EXPIRED: &str = "EXPIRED";

// ---------------------------------------------------------------------------
// Strongly-typed status classifications (returned by the helpers; the raw
// wire string is always preserved via the `Other` variants and the source
// result struct). Classification is infallible (unknown -> `Other`), so it is
// exposed via `From<&str>` rather than a fallible `from_str`.
// ---------------------------------------------------------------------------

/// Classified `orderStatus` of a payment notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    PayInProgress,
    AuthorizationRequired,
    AuthedWaitingCapture,
    PaySuccess,
    OrderClose,
    /// Any status string the SDK does not model (forward-compat).
    Other(String),
}

impl From<&str> for OrderStatus {
    fn from(s: &str) -> Self {
        match s {
            ORDER_STATUS_PAY_IN_PROGRESS => OrderStatus::PayInProgress,
            ORDER_STATUS_AUTHORIZATION_REQUIRED => OrderStatus::AuthorizationRequired,
            ORDER_STATUS_AUTHED_WAITING_CAPTURE => OrderStatus::AuthedWaitingCapture,
            ORDER_STATUS_PAY_SUCCESS => OrderStatus::PaySuccess,
            ORDER_STATUS_ORDER_CLOSE => OrderStatus::OrderClose,
            other => OrderStatus::Other(other.to_string()),
        }
    }
}

impl OrderStatus {
    /// The canonical wire string for this status.
    pub fn as_str(&self) -> &str {
        match self {
            OrderStatus::PayInProgress => ORDER_STATUS_PAY_IN_PROGRESS,
            OrderStatus::AuthorizationRequired => ORDER_STATUS_AUTHORIZATION_REQUIRED,
            OrderStatus::AuthedWaitingCapture => ORDER_STATUS_AUTHED_WAITING_CAPTURE,
            OrderStatus::PaySuccess => ORDER_STATUS_PAY_SUCCESS,
            OrderStatus::OrderClose => ORDER_STATUS_ORDER_CLOSE,
            OrderStatus::Other(s) => s.as_str(),
        }
    }
}

/// Classified `refundStatus` of a refund notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefundStatus {
    InProgress,
    PartiallyRefunded,
    FullyRefunded,
    Failed,
    /// Any status string the SDK does not model (forward-compat).
    Other(String),
}

impl From<&str> for RefundStatus {
    fn from(s: &str) -> Self {
        match s {
            REFUND_STATUS_IN_PROGRESS => RefundStatus::InProgress,
            REFUND_STATUS_PARTIALLY_REFUNDED => RefundStatus::PartiallyRefunded,
            REFUND_STATUS_FULLY_REFUNDED => RefundStatus::FullyRefunded,
            REFUND_STATUS_FAILED => RefundStatus::Failed,
            other => RefundStatus::Other(other.to_string()),
        }
    }
}

impl RefundStatus {
    /// The canonical wire string for this status.
    pub fn as_str(&self) -> &str {
        match self {
            RefundStatus::InProgress => REFUND_STATUS_IN_PROGRESS,
            RefundStatus::PartiallyRefunded => REFUND_STATUS_PARTIALLY_REFUNDED,
            RefundStatus::FullyRefunded => REFUND_STATUS_FULLY_REFUNDED,
            RefundStatus::Failed => REFUND_STATUS_FAILED,
            RefundStatus::Other(s) => s.as_str(),
        }
    }
}

/// Classified `subscriptionStatus` of a subscription status notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionStatus {
    AuthorizationRequired,
    InProgress,
    Active,
    Close,
    MerchantCancelled,
    UserCancelled,
    ChannelCancelled,
    Expired,
    /// Any status string the SDK does not model (forward-compat).
    Other(String),
}

impl From<&str> for SubscriptionStatus {
    fn from(s: &str) -> Self {
        match s {
            SUBSCRIPTION_STATUS_AUTHORIZATION_REQUIRED => SubscriptionStatus::AuthorizationRequired,
            SUBSCRIPTION_STATUS_IN_PROGRESS => SubscriptionStatus::InProgress,
            SUBSCRIPTION_STATUS_ACTIVE => SubscriptionStatus::Active,
            SUBSCRIPTION_STATUS_CLOSE => SubscriptionStatus::Close,
            SUBSCRIPTION_STATUS_MERCHANT_CANCELLED => SubscriptionStatus::MerchantCancelled,
            SUBSCRIPTION_STATUS_USER_CANCELLED => SubscriptionStatus::UserCancelled,
            SUBSCRIPTION_STATUS_CHANNEL_CANCELLED => SubscriptionStatus::ChannelCancelled,
            SUBSCRIPTION_STATUS_EXPIRED => SubscriptionStatus::Expired,
            other => SubscriptionStatus::Other(other.to_string()),
        }
    }
}

impl SubscriptionStatus {
    /// The canonical wire string for this status.
    pub fn as_str(&self) -> &str {
        match self {
            SubscriptionStatus::AuthorizationRequired => SUBSCRIPTION_STATUS_AUTHORIZATION_REQUIRED,
            SubscriptionStatus::InProgress => SUBSCRIPTION_STATUS_IN_PROGRESS,
            SubscriptionStatus::Active => SUBSCRIPTION_STATUS_ACTIVE,
            SubscriptionStatus::Close => SUBSCRIPTION_STATUS_CLOSE,
            SubscriptionStatus::MerchantCancelled => SUBSCRIPTION_STATUS_MERCHANT_CANCELLED,
            SubscriptionStatus::UserCancelled => SUBSCRIPTION_STATUS_USER_CANCELLED,
            SubscriptionStatus::ChannelCancelled => SUBSCRIPTION_STATUS_CHANNEL_CANCELLED,
            SubscriptionStatus::Expired => SUBSCRIPTION_STATUS_EXPIRED,
            SubscriptionStatus::Other(s) => s.as_str(),
        }
    }
}

// ---------------------------------------------------------------------------
// The parsed webhook event.
// ---------------------------------------------------------------------------

/// A verified, parsed webhook event. The user matches on this in their own
/// endpoint handler — the SDK holds no dispatch state and no handler registry.
#[derive(Debug, Clone)]
pub enum WebhookEvent {
    /// `PAYMENT_NOTIFICATION`.
    Payment(PaymentNotificationResult),
    /// `REFUND_NOTIFICATION`.
    Refund(RefundNotificationResult),
    /// `SUBSCRIPTION_STATUS_NOTIFICATION`. Carries both subscription-lifecycle
    /// and subscription-payment notifications (see
    /// [`WebhookEvent::subscription_dispatch`]).
    SubscriptionStatus(SubscriptionNotificationResult),
    /// `SUBSCRIPTION_PERIOD_CHANGED_NOTIFICATION`.
    SubscriptionPeriodChanged(SubscriptionNotificationResult),
    /// `SUBSCRIPTION_CHANGE_NOTIFICATION`.
    SubscriptionChange(SubscriptionChangeNotificationResult),
}

impl WebhookEvent {
    /// The canonical `eventType` string for this event.
    pub fn event_type(&self) -> &'static str {
        match self {
            WebhookEvent::Payment(_) => EVENT_PAYMENT,
            WebhookEvent::Refund(_) => EVENT_REFUND,
            WebhookEvent::SubscriptionStatus(_) => EVENT_SUBSCRIPTION_STATUS,
            WebhookEvent::SubscriptionPeriodChanged(_) => EVENT_SUBSCRIPTION_PERIOD_CHANGED,
            WebhookEvent::SubscriptionChange(_) => EVENT_SUBSCRIPTION_CHANGE,
        }
    }

    /// Classify a payment event by its `orderStatus`. Returns `None` for any
    /// non-payment event.
    pub fn order_status(&self) -> Option<OrderStatus> {
        match self {
            WebhookEvent::Payment(r) => Some(OrderStatus::from(r.order_status.as_str())),
            _ => None,
        }
    }

    /// Classify a refund event by its `refundStatus`. Returns `None` for any
    /// non-refund event.
    pub fn refund_status(&self) -> Option<RefundStatus> {
        match self {
            WebhookEvent::Refund(r) => Some(RefundStatus::from(r.refund_status.as_str())),
            _ => None,
        }
    }

    /// Classify a subscription-status event by its `subscriptionStatus`. Returns
    /// `None` for any non-`SubscriptionStatus` event. (Use the source result on
    /// [`WebhookEvent::SubscriptionPeriodChanged`] directly if you need its
    /// status — period-changed events are not classified here.)
    pub fn subscription_status(&self) -> Option<SubscriptionStatus> {
        match self {
            WebhookEvent::SubscriptionStatus(r) => {
                Some(SubscriptionStatus::from(r.subscription_status.as_str()))
            }
            _ => None,
        }
    }

    /// Reproduce Go's `SUBSCRIPTION_STATUS_NOTIFICATION` routing semantics:
    /// "status takes precedence over payment, payment as fallback".
    ///
    /// In Go, a single `SUBSCRIPTION_STATUS_NOTIFICATION` event feeds two
    /// possible handlers — `OnSubscriptionStatus` (priority) and
    /// `OnSubscriptionPayment` (fallback, used only when no status handler is
    /// registered). The Rust port has no handler registry, so this helper lets
    /// the caller express the same precedence: pass which handlers you have, and
    /// it tells you which one to invoke for a [`WebhookEvent::SubscriptionStatus`]
    /// event. Returns `None` for any other event, or when neither handler is
    /// available.
    pub fn subscription_dispatch(
        &self,
        has_status_handler: bool,
        has_payment_handler: bool,
    ) -> Option<SubscriptionDispatch<'_>> {
        match self {
            WebhookEvent::SubscriptionStatus(r) => {
                if has_status_handler {
                    Some(SubscriptionDispatch::Status(r))
                } else if has_payment_handler {
                    Some(SubscriptionDispatch::Payment(r))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Outcome of [`WebhookEvent::subscription_dispatch`]: which logical handler a
/// `SUBSCRIPTION_STATUS_NOTIFICATION` should be routed to under Go's
/// status-over-payment precedence. Both variants borrow the same underlying
/// result.
#[derive(Debug, Clone)]
pub enum SubscriptionDispatch<'a> {
    /// Route to the subscription *status* handler (priority).
    Status(&'a SubscriptionNotificationResult),
    /// Route to the subscription *payment* handler (fallback).
    Payment(&'a SubscriptionNotificationResult),
}
