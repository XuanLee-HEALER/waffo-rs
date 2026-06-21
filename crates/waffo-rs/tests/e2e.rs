//! End-to-end tests against the real Waffo **sandbox**.
//!
//! Every test is `#[ignore]`d: they need real credentials (read from the
//! environment via [`WaffoConfig::from_env`], with a local `.env` loaded by
//! `dotenvy`) and live network, so they do not run in the normal suite, CI, or
//! coverage. Run them explicitly:
//!
//! ```sh
//! cargo test --test e2e -- --ignored --nocapture
//! ```
//! (or `just e2e`). Webhooks are intentionally out of scope here.
//!
//! ## What "passing" means
//!
//! These tests validate that the SDK completes the request → signed response →
//! verify → decode round-trip against the real server. A *business* outcome
//! (server `code != "0"`) still counts as a successful round-trip — only an
//! **SDK-level** failure (transport / serialization / signature / config) fails
//! the test. The amount-driven error-simulation tests additionally assert the
//! exact mapped error (see `docs/INTEGRATION.md`). Completing a payment requires
//! the hosted cashier (browser), so `capture` / happy-path `refund` are
//! exercised for plumbing only — a real refund needs a prerequisite paid order.

use std::sync::atomic::{AtomicU64, Ordering};

use waffo_rs::biz::{merchant, order, refund, subscription};
use waffo_rs::{Client, WaffoConfig, WaffoError};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn client_from_env() -> Client {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = dotenvy::dotenv();
    });

    let cfg = WaffoConfig::from_env().expect(
        "e2e: set WAFFO_MERCHANT_API_KEY / WAFFO_MERCHANT_PRIVATE_KEY / WAFFO_PUBLIC_KEY \
         (+ WAFFO_MERCHANT_ID, WAFFO_ENVIRONMENT=SANDBOX) — e.g. in a .env file",
    );
    Client::new(cfg).expect("e2e: client should build from the env config")
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A short (<= 32 char) process-unique id. The API caps several id fields at 32.
fn unique(prefix: &str) -> String {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{prefix}{ms}{n}")
}

/// True when the error is an SDK-level failure (not a server business outcome).
fn is_sdk_failure(err: &WaffoError) -> bool {
    !matches!(
        err,
        WaffoError::Api { .. } | WaffoError::UnknownStatus { .. }
    )
}

/// Assert the SDK completed the round-trip. `Ok` or a server business error both
/// pass; only an SDK-level failure panics.
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
// param builders (merchant_info + *requested_at are auto-injected; payMethodType
// is left unset so the cashier offers every method the merchant signed for)
// ---------------------------------------------------------------------------

fn create_order_params(rid: &str, oid: &str, amount: &str) -> order::CreateOrderParams {
    order::CreateOrderParams {
        payment_request_id: rid.to_string(),
        merchant_order_id: oid.to_string(),
        order_currency: "USD".to_string(),
        order_amount: amount.to_string(),
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
        goods_info: Some(order::GoodsInfo {
            goods_id: Some("e2e_goods".to_string()),
            goods_name: Some("E2E Goods".to_string()),
            goods_category: Some("GOODS".to_string()),
            goods_url: Some("https://example.com/goods".to_string()),
            goods_quantity: Some(1),
            ..Default::default()
        }),
        payment_info: Some(order::PaymentInfo {
            product_name: Some("ONE_TIME_PAYMENT".to_string()),
            pay_method_name: Some("CC_VISA".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn create_subscription_params(sr: &str, msid: &str) -> subscription::CreateSubscriptionParams {
    subscription::CreateSubscriptionParams {
        subscription_request: sr.to_string(),
        merchant_subscription_id: msid.to_string(),
        currency: "USD".to_string(),
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
        goods_info: Some(subscription::SubscriptionGoodsInfo {
            goods_id: Some("e2e_goods".to_string()),
            goods_name: Some("E2E Subscription".to_string()),
            goods_category: Some("SUBSCRIPTION".to_string()),
            goods_url: Some("https://example.com/sub".to_string()),
            goods_quantity: Some(1),
            ..Default::default()
        }),
        payment_info: Some(subscription::SubscriptionPaymentInfo {
            product_name: Some("SUBSCRIPTION".to_string()),
            pay_method_name: Some("CC_VISA".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// config (pure reads — succeed against a valid sandbox)
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
// order: create -> inquiry -> cancel (positive flow up to the cashier)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires sandbox credentials via env; run with --ignored"]
async fn e2e_order_lifecycle() {
    let client = client_from_env();
    let rid = unique("po");
    let oid = unique("PO");

    let created = order::create(&client, create_order_params(&rid, &oid, "10.00"), None).await;
    if let Ok(d) = &created {
        println!(
            "[order::create] status={:?} acquiringOrderId={:?} redirect={}",
            d.order_status,
            d.acquiring_order_id,
            d.fetch_redirect_url()
        );
    }
    assert_round_trip("order::create", created);

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

    assert_round_trip(
        "order::cancel",
        order::cancel(
            &client,
            order::CancelOrderParams {
                payment_request_id: Some(rid),
                ..Default::default()
            },
            None,
        )
        .await,
    );
}

// ---------------------------------------------------------------------------
// order error simulations (sandbox triggers via orderAmount; see INTEGRATION.md)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires sandbox credentials via env; run with --ignored"]
async fn e2e_order_error_simulations() {
    let client = client_from_env();

    // orderAmount 9.2/92/992/... -> E0001 Unknown Status -> WaffoError::UnknownStatus
    let err = order::create(
        &client,
        create_order_params(&unique("po"), &unique("PO"), "992"),
        None,
    )
    .await
    .expect_err("amount 992 should be a business error");
    println!("[sim E0001] {err}");
    assert!(
        err.is_unknown_status(),
        "amount 992 should map to UnknownStatus (E0001); got {err:?}"
    );

    // orderAmount 9/90/990/... -> C0005 Payment Channel Rejection
    let err = order::create(
        &client,
        create_order_params(&unique("po"), &unique("PO"), "990"),
        None,
    )
    .await
    .expect_err("amount 990 should be a business error");
    println!("[sim C0005] {err}");
    assert_eq!(err.api_code(), Some("C0005"), "amount 990 should be C0005");

    // orderAmount 9.1/91/991/... -> C0001 System Error
    let err = order::create(
        &client,
        create_order_params(&unique("po"), &unique("PO"), "991"),
        None,
    )
    .await
    .expect_err("amount 991 should be a business error");
    println!("[sim C0001] {err}");
    assert_eq!(err.api_code(), Some("C0001"), "amount 991 should be C0001");
}

// ---------------------------------------------------------------------------
// refund / capture / inquiry-refund (need a paid / authorized order -> plumbing)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires sandbox credentials via env; run with --ignored"]
async fn e2e_refund_and_capture() {
    let client = client_from_env();
    let rid = unique("po");
    let oid = unique("PO");

    // Create an (unpaid) order to obtain a real acquiringOrderId.
    let created = order::create(&client, create_order_params(&rid, &oid, "10.00"), None).await;
    let acquiring_id = created
        .as_ref()
        .ok()
        .and_then(|d| d.acquiring_order_id.clone())
        .unwrap_or_default();
    assert_round_trip("order::create", created);

    // Refunding an unpaid order -> business error (no successful payment yet).
    assert_round_trip(
        "order::refund",
        order::refund(
            &client,
            order::RefundOrderParams {
                refund_request_id: unique("rf"),
                acquiring_order_id: acquiring_id.clone(),
                refund_amount: "10.00".to_string(),
                refund_reason: "e2e".to_string(),
                ..Default::default()
            },
            None,
        )
        .await,
    );

    // Capturing without an AUTHED_WAITING_CAPTURE order -> business error.
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

    assert_round_trip(
        "refund::inquiry",
        refund::inquiry(
            &client,
            refund::InquiryRefundParams {
                refund_request_id: unique("rf"),
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
async fn e2e_subscription_lifecycle() {
    let client = client_from_env();
    let sr = unique("so");
    let msid = unique("SO");

    let created = subscription::create(&client, create_subscription_params(&sr, &msid), None).await;
    if let Ok(d) = &created {
        println!(
            "[subscription::create] status={:?} subscriptionId={:?} redirect={}",
            d.subscription_status,
            d.subscription_id,
            d.fetch_redirect_url()
        );
    }
    let subscription_id = created
        .as_ref()
        .ok()
        .and_then(|d| d.subscription_id.clone())
        .unwrap_or_default();
    assert_round_trip("subscription::create", created);

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
    let origin_sr = unique("so");
    let new_sr = unique("sn");

    assert_round_trip(
        "subscription::change",
        subscription::change(
            &client,
            subscription::ChangeSubscriptionParams {
                subscription_request: new_sr.clone(),
                merchant_subscription_id: Some(unique("SC")),
                origin_subscription_request: origin_sr.clone(),
                remaining_amount: "50.00".to_string(),
                currency: "USD".to_string(),
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
                goods_info: Some(subscription::SubscriptionGoodsInfo {
                    goods_id: Some("e2e_goods".to_string()),
                    goods_name: Some("E2E Subscription".to_string()),
                    goods_category: Some("SUBSCRIPTION".to_string()),
                    goods_quantity: Some(1),
                    ..Default::default()
                }),
                payment_info: Some(subscription::SubscriptionPaymentInfo {
                    product_name: Some("SUBSCRIPTION".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
        )
        .await,
    );

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
