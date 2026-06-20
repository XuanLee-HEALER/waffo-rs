//! Unit tests for the `fetch_redirect_url` helpers, which parse the
//! string-encoded `*Action` fields on order / subscription responses.

use waffo_rs::biz::order::CreateOrderData;
use waffo_rs::biz::subscription::{ChangeSubscriptionData, CreateSubscriptionData};

// ---- order ----------------------------------------------------------------

#[test]
fn order_redirect_empty_or_unparseable_is_blank() {
    assert_eq!(CreateOrderData::default().fetch_redirect_url(), "");

    let bad = CreateOrderData {
        order_action: Some("not json".to_string()),
        ..Default::default()
    };
    assert_eq!(bad.fetch_redirect_url(), "");
}

#[test]
fn order_redirect_prefers_deeplink_for_deeplink_action() {
    let d = CreateOrderData {
        order_action: Some(
            r#"{"actionType":"DEEPLINK","webUrl":"https://w","deeplinkUrl":"app://d"}"#.to_string(),
        ),
        ..Default::default()
    };
    assert_eq!(d.fetch_redirect_url(), "app://d");
}

#[test]
fn order_redirect_falls_back_to_web_url() {
    let redirect = CreateOrderData {
        order_action: Some(r#"{"actionType":"REDIRECT","webUrl":"https://w"}"#.to_string()),
        ..Default::default()
    };
    assert_eq!(redirect.fetch_redirect_url(), "https://w");

    // DEEPLINK action but empty deeplink -> web url.
    let empty_deeplink = CreateOrderData {
        order_action: Some(
            r#"{"actionType":"DEEPLINK","webUrl":"https://w","deeplinkUrl":""}"#.to_string(),
        ),
        ..Default::default()
    };
    assert_eq!(empty_deeplink.fetch_redirect_url(), "https://w");
}

// ---- subscription: create (webUrl, then deeplinkUrl) ----------------------

#[test]
fn subscription_create_redirect_variants() {
    assert_eq!(CreateSubscriptionData::default().fetch_redirect_url(), "");

    // A bare URL is returned directly.
    let direct = CreateSubscriptionData {
        subscription_action: Some("https://pay".to_string()),
        ..Default::default()
    };
    assert_eq!(direct.fetch_redirect_url(), "https://pay");

    // JSON action: webUrl wins.
    let web = CreateSubscriptionData {
        subscription_action: Some(r#"{"webUrl":"https://w","deeplinkUrl":"app://d"}"#.to_string()),
        ..Default::default()
    };
    assert_eq!(web.fetch_redirect_url(), "https://w");

    // JSON action: deeplinkUrl is the fallback when webUrl is empty.
    let deep = CreateSubscriptionData {
        subscription_action: Some(r#"{"deeplinkUrl":"app://d"}"#.to_string()),
        ..Default::default()
    };
    assert_eq!(deep.fetch_redirect_url(), "app://d");

    let bad = CreateSubscriptionData {
        subscription_action: Some("garbage".to_string()),
        ..Default::default()
    };
    assert_eq!(bad.fetch_redirect_url(), "");
}

// ---- subscription: change (webUrl only) -----------------------------------

#[test]
fn subscription_change_redirect_web_only() {
    assert_eq!(ChangeSubscriptionData::default().fetch_redirect_url(), "");

    let direct = ChangeSubscriptionData {
        subscription_action: Some("http://pay".to_string()),
        ..Default::default()
    };
    assert_eq!(direct.fetch_redirect_url(), "http://pay");

    let web = ChangeSubscriptionData {
        subscription_action: Some(r#"{"webUrl":"https://w"}"#.to_string()),
        ..Default::default()
    };
    assert_eq!(web.fetch_redirect_url(), "https://w");

    // change only extracts webUrl; a deeplink-only action yields "".
    let deep = ChangeSubscriptionData {
        subscription_action: Some(r#"{"deeplinkUrl":"app://d"}"#.to_string()),
        ..Default::default()
    };
    assert_eq!(deep.fetch_redirect_url(), "");
}
