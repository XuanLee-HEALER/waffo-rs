//! End-to-end tests against the real Waffo **sandbox**.
//!
//! Every test is `#[ignore]`d: they need real credentials (read from the
//! environment via [`WaffoConfig::from_env`]) and live network, so they do not
//! run in the normal suite, CI, or coverage. Run them explicitly:
//!
//! ```sh
//! WAFFO_API_KEY=... WAFFO_PRIVATE_KEY=... WAFFO_PUBLIC_KEY=... \
//! WAFFO_MERCHANT_ID=... WAFFO_ENVIRONMENT=SANDBOX \
//! cargo test --test e2e -- --ignored --nocapture
//! ```
//! (or `just e2e`).
//!
//! Webhooks are intentionally out of scope here.
//!
//! ## What "passing" means
//!
//! These tests validate that the SDK can complete the request → signed
//! response → verify → decode round-trip against the real server for every
//! endpoint. A *business* outcome (server `code != "0"`, e.g. refunding an
//! unpaid order) still counts as a successful round-trip — only an **SDK-level**
//! failure (transport / serialization / signature / config) fails the test.
//! Steps that require browser checkout (completing a payment, capturing an
//! authorized card order) cannot be automated, so those endpoints are exercised
//! for plumbing only.

use std::sync::atomic::{AtomicU64, Ordering};

use waffo_rs::biz::{merchant, order, refund, subscription};
use waffo_rs::{Client, WaffoConfig, WaffoError};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn client_from_env() -> Client {
    // Load a local .env file (workspace root, gitignored) once, if present.
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = dotenvy::dotenv();
    });

    let cfg = WaffoConfig::from_env().expect(
        "e2e: set WAFFO_API_KEY / WAFFO_PRIVATE_KEY / WAFFO_PUBLIC_KEY \
         (+ WAFFO_MERCHANT_ID, WAFFO_ENVIRONMENT=SANDBOX) — e.g. in a .env file",
    );
    Client::new(cfg).expect("e2e: client should build from the env config")
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A process-unique id with the given prefix.
fn unique(prefix: &str) -> String {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{prefix}_{nanos}_{n}")
}

/// True when the error is an SDK-level failure (not a server business outcome).
fn is_sdk_failure(err: &WaffoError) -> bool {
    !matches!(
        err,
        WaffoError::Api { .. } | WaffoError::UnknownStatus { .. }
    )
}

/// Assert the SDK completed the round-trip. `Ok` or a server business error
/// both pass; only an SDK-level failure panics.
fn assert_round_trip<T: std::fmt::Debug>(label: &str, result: waffo_rs::Result<T>) {
    match result {
        Ok(data) => println!("[{label}] ok: {data:?}"),
        Err(ref e) if !is_sdk_failure(e) => println!("[{label}] business response: {e}"),
        Err(e) => panic!("[{label}] SDK-level failure (round-trip did not complete): {e:?}"),
    }
}

fn merchant_id(client: &Client) -> String {
    client.config().merchant_id.clone().unwrap_or_default()
}

// ---------------------------------------------------------------------------
// param builders (merchant_info + *requested_at are auto-injected)
// ---------------------------------------------------------------------------

fn create_order_params(rid: &str, oid: &str) -> order::CreateOrderParams {
    order::CreateOrderParams {
        payment_request_id: rid.to_string(),
        merchant_order_id: oid.to_string(),
        order_currency: "HKD".to_string(),
        order_amount: "10.00".to_string(),
        order_description: "waffo-rs e2e order".to_string(),
        notify_url: "https://httpbin.org/post".to_string(),
        success_redirect_url: Some("https://httpbin.org/get?status=success".to_string()),
        failed_redirect_url: Some("https://httpbin.org/get?status=failed".to_string()),
        cancel_redirect_url: Some("https://httpbin.org/get?status=cancel".to_string()),
        user_info: Some(order::UserInfo {
            user_id: Some("e2e_user".to_string()),
            user_email: Some("e2e@test.com".to_string()),
            user_terminal: Some("WEB".to_string()),
            ..Default::default()
        }),
        payment_info: Some(order::PaymentInfo {
            product_name: Some("ONE_TIME_PAYMENT".to_string()),
            pay_method_type: Some("CREDITCARD".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn create_subscription_params(sr: &str, msid: &str) -> subscription::CreateSubscriptionParams {
    subscription::CreateSubscriptionParams {
        subscription_request: sr.to_string(),
        merchant_subscription_id: msid.to_string(),
        currency: "HKD".to_string(),
        amount: "99.00".to_string(),
        notify_url: "https://httpbin.org/post".to_string(),
        success_redirect_url: Some("https://httpbin.org/get?status=success".to_string()),
        failed_redirect_url: Some("https://httpbin.org/get?status=failed".to_string()),
        cancel_redirect_url: Some("https://httpbin.org/get?status=cancel".to_string()),
        product_info: Some(subscription::ProductInfo {
            description: "waffo-rs e2e subscription".to_string(),
            period_type: "MONTHLY".to_string(),
            period_interval: "1".to_string(),
            number_of_period: Some("12".to_string()),
            ..Default::default()
        }),
        user_info: Some(subscription::SubscriptionUserInfo {
            user_id: Some("e2e_user".to_string()),
            user_email: Some("e2e@test.com".to_string()),
            user_terminal: Some("WEB".to_string()),
            ..Default::default()
        }),
        payment_info: Some(subscription::SubscriptionPaymentInfo {
            product_name: Some("SUBSCRIPTION".to_string()),
            pay_method_type: Some("CREDITCARD".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// config (pure reads — should succeed against a valid sandbox)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires sandbox credentials via env; run with --ignored"]
async fn e2e_merchant_and_pay_method_config() {
    let client = client_from_env();
    let mid = merchant_id(&client);

    assert_round_trip(
        "merchant::merchant_config_inquiry",
        merchant::merchant_config_inquiry(
            &client,
            merchant::InquiryMerchantConfigParams {
                merchant_id: mid.clone(),
                ..Default::default()
            },
            None,
        )
        .await,
    );

    assert_round_trip(
        "merchant::pay_method_config_inquiry",
        merchant::pay_method_config_inquiry(
            &client,
            merchant::InquiryPayMethodConfigParams {
                merchant_id: mid,
                ..Default::default()
            },
            None,
        )
        .await,
    );
}

// ---------------------------------------------------------------------------
// order: create -> inquiry -> cancel -> refund -> capture
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires sandbox credentials via env; run with --ignored"]
async fn e2e_order_endpoints() {
    let client = client_from_env();
    let rid = unique("e2e_pay");
    let oid = unique("E2E_ORDER");

    // create
    let created = order::create(&client, create_order_params(&rid, &oid), None).await;
    let acquiring_id = match &created {
        Ok(d) => d.acquiring_order_id.clone().unwrap_or_default(),
        Err(_) => String::new(),
    };
    assert_round_trip("order::create", created);

    // inquiry (by the payment request id we chose)
    assert_round_trip(
        "order::inquiry",
        order::inquiry(
            &client,
            order::InquiryOrderParams {
                payment_request_id: Some(rid.clone()),
                ..Default::default()
            },
            None,
        )
        .await,
    );

    // cancel (an unpaid order is cancellable; otherwise a business response)
    assert_round_trip(
        "order::cancel",
        order::cancel(
            &client,
            order::CancelOrderParams {
                payment_request_id: Some(rid.clone()),
                ..Default::default()
            },
            None,
        )
        .await,
    );

    // refund (needs a paid order -> expect a business response, plumbing only)
    assert_round_trip(
        "order::refund",
        order::refund(
            &client,
            order::RefundOrderParams {
                refund_request_id: unique("e2e_refund"),
                acquiring_order_id: acquiring_id.clone(),
                refund_amount: "10.00".to_string(),
                refund_reason: "e2e".to_string(),
                ..Default::default()
            },
            None,
        )
        .await,
    );

    // capture (needs an authorized card order -> business response, plumbing only)
    assert_round_trip(
        "order::capture",
        order::capture(
            &client,
            order::CaptureOrderParams {
                payment_request_id: Some(rid),
                acquiring_order_id: Some(acquiring_id),
                capture_amount: "10.00".to_string(),
                ..Default::default()
            },
            None,
        )
        .await,
    );
}

#[tokio::test]
#[ignore = "requires sandbox credentials via env; run with --ignored"]
async fn e2e_refund_inquiry() {
    let client = client_from_env();
    // No such refund exists -> business response, exercises the plumbing.
    assert_round_trip(
        "refund::inquiry",
        refund::inquiry(
            &client,
            refund::InquiryRefundParams {
                refund_request_id: unique("e2e_refund"),
                ..Default::default()
            },
            None,
        )
        .await,
    );
}

// ---------------------------------------------------------------------------
// subscription: create -> inquiry -> manage -> update -> cancel
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires sandbox credentials via env; run with --ignored"]
async fn e2e_subscription_endpoints() {
    let client = client_from_env();
    let sr = unique("e2e_sub");
    let msid = unique("E2E_SUB");

    // create
    let created = subscription::create(&client, create_subscription_params(&sr, &msid), None).await;
    let subscription_id = match &created {
        Ok(d) => d.subscription_id.clone().unwrap_or_default(),
        Err(_) => String::new(),
    };
    assert_round_trip("subscription::create", created);

    // inquiry
    assert_round_trip(
        "subscription::inquiry",
        subscription::inquiry(
            &client,
            subscription::InquirySubscriptionParams {
                subscription_request: Some(sr.clone()),
                ..Default::default()
            },
            None,
        )
        .await,
    );

    // manage (management URL)
    assert_round_trip(
        "subscription::manage",
        subscription::manage(
            &client,
            subscription::ManageSubscriptionParams {
                subscription_id: Some(subscription_id.clone()),
                subscription_request: Some(sr.clone()),
                ..Default::default()
            },
            None,
        )
        .await,
    );

    // update (amount / scheduled amounts; needs an active subscription)
    assert_round_trip(
        "subscription::update",
        subscription::update(
            &client,
            subscription::UpdateSubscriptionParams {
                subscription_request: Some(sr),
                subscription_id: Some(subscription_id.clone()),
                amount: Some("199.00".to_string()),
                ..Default::default()
            },
            None,
        )
        .await,
    );

    // cancel
    assert_round_trip(
        "subscription::cancel",
        subscription::cancel(
            &client,
            subscription::CancelSubscriptionParams {
                subscription_id,
                ..Default::default()
            },
            None,
        )
        .await,
    );
}

// ---------------------------------------------------------------------------
// subscription change: change -> change_inquiry
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires sandbox credentials via env; run with --ignored"]
async fn e2e_subscription_change() {
    let client = client_from_env();
    let origin_sr = unique("e2e_orig_sub");
    let new_sr = unique("e2e_new_sub");

    // change (needs an existing origin subscription -> business response)
    assert_round_trip(
        "subscription::change",
        subscription::change(
            &client,
            subscription::ChangeSubscriptionParams {
                subscription_request: new_sr.clone(),
                merchant_subscription_id: Some(unique("E2E_CHANGE")),
                origin_subscription_request: origin_sr.clone(),
                remaining_amount: "50.00".to_string(),
                currency: "HKD".to_string(),
                notify_url: "https://httpbin.org/post".to_string(),
                product_info_list: vec![subscription::SubscriptionChangeProductInfo {
                    description: "waffo-rs e2e upgrade".to_string(),
                    period_type: "MONTHLY".to_string(),
                    period_interval: "1".to_string(),
                    amount: "199.00".to_string(),
                    number_of_period: Some("12".to_string()),
                    ..Default::default()
                }],
                user_info: Some(subscription::SubscriptionUserInfo {
                    user_id: Some("e2e_user".to_string()),
                    user_email: Some("e2e@test.com".to_string()),
                    ..Default::default()
                }),
                payment_info: Some(subscription::SubscriptionPaymentInfo {
                    product_name: Some("SUBSCRIPTION".to_string()),
                    pay_method_type: Some("CREDITCARD".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
        )
        .await,
    );

    // change inquiry (needs both request ids)
    assert_round_trip(
        "subscription::change_inquiry",
        subscription::change_inquiry(
            &client,
            subscription::ChangeInquiryParams {
                origin_subscription_request: origin_sr,
                subscription_request: new_sr,
                ..Default::default()
            },
            None,
        )
        .await,
    );
}
