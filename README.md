# waffo-rs

[English](README.md) · [中文](README.zh-CN.md)

[![crates.io](https://img.shields.io/crates/v/waffo-rs)](https://crates.io/crates/waffo-rs)
[![docs.rs](https://img.shields.io/docsrs/waffo-rs)](https://docs.rs/waffo-rs)
![coverage](https://img.shields.io/badge/coverage-~95%25-brightgreen)
![rust](https://img.shields.io/badge/rust-1.96%2B-orange)
![license](https://img.shields.io/badge/license-MIT-blue)

Async Rust SDK for the [Waffo](https://waffo.com) payment platform — orders,
refunds, subscriptions, chargebacks, merchant configuration and webhooks, with
automatic request signing and response/webhook signature verification.

This is a Rust port of the official Go SDK (`waffo-go`). It is **not a 1:1
translation** — it follows the same wire protocol and domain model, but is
shaped to be idiomatic and ergonomic for the Rust async ecosystem.

> **Status: v1.0.** The full API surface is implemented, documented and tested —
> unit + `wiremock` transport tests, plus end-to-end tests against the live
> sandbox covering every endpoint and all six webhook events over a real tunnel.

## Highlights

- **Async-only**, built on [`reqwest`](https://docs.rs/reqwest) /
  [`tokio`](https://tokio.rs).
- **One uniform request path.** Every endpoint is a small `Endpoint`
  declaration (request/response types + path); a single generic `send` does the
  inject → serialize → sign → send → verify → envelope → error-map pipeline for
  all of them.
- **Automatic field injection** (`merchantId`, `requestedAt`) via a
  `#[derive(WaffoRequest)]` proc-macro — replacing the Go SDK's reflection.
- **Strong types that mirror the Go SDK** byte-for-byte (JSON tags), plus an
  escape hatch: `extraParams` on requests and a `#[serde(flatten)]` catch-all on
  responses so new server fields never break deserialization.
- **A single `WaffoError`** (`Result<T, WaffoError>`). Server `code != "0"`
  becomes a business error; `E0001` (and read-method transport failures) become
  `UnknownStatus` — telling you to re-inquire rather than assume failure.
- **Webhooks without a registry.** Verify + parse the raw body into a
  `WebhookEvent` enum you `match` on, then reply with a three-state signed ack
  (`WebhookAck::{Success, Failed, Unknown}` — `Failed`/`Unknown` make Waffo
  retry, up to 24h). Optional thin `axum` integration behind a feature flag.

## Requirements

- Rust 1.96+ (edition 2024)
- A Tokio runtime (the SDK does not pull in a runtime itself)

## Installation

```toml
[dependencies]
waffo-rs = "1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Enable the optional `axum` integration with:

```toml
waffo-rs = { version = "1.0", features = ["axum"] }
```

## Quick start

### Configure a client

```rust
use waffo_rs::{Client, Environment, WaffoConfig};

let config = WaffoConfig::builder()
    .api_key("YOUR_API_KEY")
    .private_key("BASE64_PKCS8_PRIVATE_KEY")     // your merchant private key
    .waffo_public_key("BASE64_X509_PUBLIC_KEY")  // Waffo's public key
    .environment(Environment::Sandbox)
    .merchant_id("YOUR_MERCHANT_ID")             // auto-injected into requests
    .build()?;

let client = Client::new(config)?;               // keys are parsed once here
# Ok::<(), waffo_rs::WaffoError>(())
```

`WaffoConfig::from_env()` reads `WAFFO_MERCHANT_API_KEY`, `WAFFO_MERCHANT_PRIVATE_KEY`,
`WAFFO_PUBLIC_KEY`, `WAFFO_ENVIRONMENT`, `WAFFO_MERCHANT_ID`.

### Create an order

```rust
use waffo_rs::biz::order::{self, CreateOrderParams, PaymentInfo, UserInfo};

let params = CreateOrderParams {
    payment_request_id: "req_1001".into(),
    merchant_order_id: "ORDER_1001".into(),
    order_currency: "USD".into(),
    order_amount: "10.00".into(),
    order_description: "T-shirt".into(),
    notify_url: "https://example.com/webhook".into(),
    user_info: Some(UserInfo { user_id: Some("u1".into()), ..Default::default() }),
    payment_info: Some(PaymentInfo {
        product_name: Some("ONE_TIME_PAYMENT".into()),
        pay_method_name: Some("CC_VISA".into()),  // must be in the merchant contract
        ..Default::default()
    }),
    ..Default::default()
};

let data = order::create(&client, params, None).await?;
println!("redirect to: {}", data.fetch_redirect_url());
# Ok::<(), waffo_rs::WaffoError>(())
```

Other resources follow the same shape:
`order::{inquiry, cancel, refund, capture}`, `refund::inquiry`,
`subscription::{create, inquiry, cancel, manage, change, change_inquiry, update}`,
`chargeback::{inquiry, update, accept, list}`,
`merchant::{merchant_config_inquiry, pay_method_config_inquiry}`. Each is
`fn(&Client, Params, Option<&RequestOptions>) -> Result<Data>`.

### Handle a webhook

Verify against the **raw request bytes** (never a re-serialized body), match the
event, and reply with a signed three-state ack. The inbound signature is the
`X-SIGNATURE` request header; the same header carries your signed reply.

```rust
use waffo_rs::webhook::{self, WebhookAck, WebhookEvent};

fn handle(client: &waffo_rs::Client, raw_body: &[u8], signature: &str)
    -> waffo_rs::Result<(String, String)>
{
    let ack = match webhook::verify_and_parse(client, raw_body, signature) {
        Ok(WebhookEvent::Payment(_p))                   => WebhookAck::Success,
        Ok(WebhookEvent::Refund(_r))                    => WebhookAck::Success,
        Ok(WebhookEvent::SubscriptionStatus(_s))        => WebhookAck::Success,
        Ok(WebhookEvent::SubscriptionPeriodChanged(_s)) => WebhookAck::Success,
        Ok(WebhookEvent::SubscriptionChange(_c))        => WebhookAck::Success,
        Ok(WebhookEvent::Chargeback(_c))                => WebhookAck::Success,
        // Bad signature / transient failure: don't acknowledge — Waffo retries.
        Err(_)                                          => WebhookAck::Failed,
    };
    // Always HTTP 200; the body ({"message":"success"|"failed"|"unknown"}) is
    // signed with your private key and decides whether Waffo retries.
    webhook::build_signed_response(client, ack)
}
```

With the `axum` feature, `waffo_rs::webhook::axum` provides
`signature_from_headers`, `parse_request` and `signed_response` helpers — thin
glue, no router or handler registry.

## Errors

All calls return `Result<T, WaffoError>`. Of note:

- `WaffoError::Api { code, message }` — server returned `code != "0"`.
- `WaffoError::UnknownStatus { .. }` — status is uncertain (`E0001`, or a
  read/idempotent call's transport failed). **Re-inquire; do not assume
  failure.** Check with `err.is_unknown_status()`.

## Project layout

```
crates/
  waffo-rs/         # the SDK (lib name: waffo_rs)
    src/
      config.rs   base.rs   crypto.rs
      common/     error + trace + null-tolerant deserialize helpers
      biz/        order / refund / subscription / chargeback / merchant
      webhook/    core + events + notifications + axum integration
  waffo-rs-derive/  # #[derive(WaffoRequest)] proc-macro
```

## Testing

```sh
cargo test                  # unit + wiremock transport tests
cargo test --features axum  # + the axum webhook integration
```

End-to-end tests against the real sandbox are `#[ignore]`d (they need
credentials via a `.env` and live network); run them with `cargo test --test e2e
-- --ignored`. RSA sign/verify is checked byte-for-byte against the Go SDK's
vectors.

Coverage is gated at **≥80% lines** via [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov)
(`just cov`, HTML report under `target/llvm-cov/`); current line coverage is
**~95%**.

## License

Licensed under the [MIT License](LICENSE-MIT).
