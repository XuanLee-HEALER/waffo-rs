//! Unit tests for the configuration layer (`waffo_rs::config`).

use waffo_rs::{Environment, RequestOptions, WaffoConfig};

#[test]
fn environment_base_urls_and_default() {
    assert_eq!(
        Environment::Sandbox.base_url(),
        "https://api-sandbox.waffo.com/api/v1"
    );
    assert_eq!(
        Environment::Production.base_url(),
        "https://api.waffo.com/api/v1"
    );
    assert_eq!(Environment::default(), Environment::Sandbox);
}

#[test]
fn builder_sets_every_field() {
    let cfg = WaffoConfig::builder()
        .api_key("KEY")
        .private_key("PRIV")
        .waffo_public_key("PUB")
        .environment(Environment::Production)
        .merchant_id("M1")
        .connect_timeout_ms(1234)
        .read_timeout_ms(5678)
        .debug_unredacted(true)
        .build()
        .expect("build should succeed");

    assert_eq!(cfg.api_key, "KEY");
    assert_eq!(cfg.private_key, "PRIV");
    assert_eq!(cfg.waffo_public_key, "PUB");
    assert_eq!(cfg.environment, Environment::Production);
    assert_eq!(cfg.merchant_id.as_deref(), Some("M1"));
    assert_eq!(cfg.connect_timeout_ms, 1234);
    assert_eq!(cfg.read_timeout_ms, 5678);
    assert!(cfg.debug_unredacted);
    assert_eq!(cfg.base_url(), "https://api.waffo.com/api/v1");
}

#[test]
fn builder_applies_defaults() {
    let cfg = WaffoConfig::builder()
        .api_key("K")
        .private_key("P")
        .waffo_public_key("W")
        .build()
        .unwrap();

    assert_eq!(cfg.environment, Environment::Sandbox);
    assert_eq!(cfg.merchant_id, None);
    assert_eq!(cfg.connect_timeout_ms, 10_000);
    assert_eq!(cfg.read_timeout_ms, 30_000);
    assert!(!cfg.debug_unredacted);
}

#[test]
fn builder_requires_api_private_and_public_keys() {
    assert!(WaffoConfig::builder().build().is_err());
    assert!(WaffoConfig::builder().api_key("K").build().is_err());
    assert!(WaffoConfig::builder()
        .api_key("K")
        .private_key("P")
        .build()
        .is_err());
}

#[test]
fn builder_treats_empty_strings_as_missing() {
    assert!(WaffoConfig::builder()
        .api_key("")
        .private_key("P")
        .waffo_public_key("W")
        .build()
        .is_err());

    let cfg = WaffoConfig::builder()
        .api_key("K")
        .private_key("P")
        .waffo_public_key("W")
        .merchant_id("")
        .build()
        .unwrap();
    assert_eq!(cfg.merchant_id, None);
}

#[test]
fn request_options_default_is_empty() {
    let o = RequestOptions::default();
    assert!(o.connect_timeout_ms.is_none());
    assert!(o.read_timeout_ms.is_none());
    assert!(o.headers.is_empty());
}

#[test]
fn from_env_reads_validates_and_errors() {
    use std::env;

    const VARS: [&str; 5] = [
        "WAFFO_API_KEY",
        "WAFFO_PRIVATE_KEY",
        "WAFFO_PUBLIC_KEY",
        "WAFFO_ENVIRONMENT",
        "WAFFO_MERCHANT_ID",
    ];

    env::set_var("WAFFO_API_KEY", "envkey");
    env::set_var("WAFFO_PRIVATE_KEY", "envpriv");
    env::set_var("WAFFO_PUBLIC_KEY", "envpub");
    env::set_var("WAFFO_ENVIRONMENT", "PRODUCTION");
    env::set_var("WAFFO_MERCHANT_ID", "envM");

    let cfg = WaffoConfig::from_env().expect("from_env should succeed");
    assert_eq!(cfg.api_key, "envkey");
    assert_eq!(cfg.private_key, "envpriv");
    assert_eq!(cfg.waffo_public_key, "envpub");
    assert_eq!(cfg.environment, Environment::Production);
    assert_eq!(cfg.merchant_id.as_deref(), Some("envM"));

    // Any non-"PRODUCTION" value falls back to Sandbox.
    env::set_var("WAFFO_ENVIRONMENT", "anything-else");
    assert_eq!(
        WaffoConfig::from_env().unwrap().environment,
        Environment::Sandbox
    );

    // A missing required variable is an error.
    env::remove_var("WAFFO_API_KEY");
    assert!(WaffoConfig::from_env().is_err());

    for v in VARS {
        env::remove_var(v);
    }
}
