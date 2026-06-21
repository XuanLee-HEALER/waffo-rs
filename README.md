# waffo-rs

Async Rust SDK for the [Waffo](https://waffo.com) payment platform — orders,
refunds, subscriptions, merchant configuration and webhooks, with automatic
request signing and response/webhook signature verification.

This is a Rust port of the official Go SDK (`waffo-go`). It is **not a 1:1
translation** — it follows the same wire protocol and domain model, but is
shaped to be idiomatic and ergonomic for the Rust async ecosystem.

> Status: early / work in progress. The API surface is complete and tested
> against the Go SDK's field definitions, but it has not been published to
> crates.io and may still change.

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
  `WebhookEvent` enum you `match` on; build the signed response. Optional thin
  `axum` integration behind a feature flag.

## Requirements

- Rust 1.75+ (edition 2021)
- A Tokio runtime (the SDK does not pull in a runtime itself)

## Installation

Not yet on crates.io — depend on it via git:

```toml
[dependencies]
waffo-rs = { git = "https://github.com/XuanLee-HEALER/waffo-rs" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Enable the optional `axum` integration with:

```toml
waffo-rs = { git = "https://github.com/XuanLee-HEALER/waffo-rs", features = ["axum"] }
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
    order_currency: "HKD".into(),
    order_amount: "10.00".into(),
    order_description: "T-shirt".into(),
    notify_url: "https://example.com/webhook".into(),
    user_info: Some(UserInfo { user_id: Some("u1".into()), ..Default::default() }),
    payment_info: Some(PaymentInfo {
        product_name: Some("ONE_TIME_PAYMENT".into()),
        pay_method_type: Some("CREDITCARD".into()),
        ..Default::default()
    }),
    ..Default::default()
};

let data = order::create(&client, params, None).await?;
println!("redirect to: {}", data.fetch_redirect_url());
# Ok::<(), waffo_rs::WaffoError>(())
```

Other resources follow the same shape: `order::{inquiry, cancel, refund, capture}`,
`refund::inquiry`, `subscription::{create, inquiry, cancel, manage, change, change_inquiry, update}`,
`merchant::{merchant_config_inquiry, pay_method_config_inquiry}`. Each is
`fn(&Client, Params, Option<&RequestOptions>) -> Result<Data>`.

### Handle a webhook

Verify against the **raw request bytes** (never a re-serialized body), match the
event, and reply with a signed response:

```rust
use waffo_rs::webhook::{self, WebhookEvent};

fn handle(client: &waffo_rs::Client, raw_body: &[u8], signature: &str)
    -> waffo_rs::Result<(String, String)>
{
    match webhook::verify_and_parse(client, raw_body, signature)? {
        WebhookEvent::Payment(p)  => { /* ... */ }
        WebhookEvent::Refund(r)   => { /* ... */ }
        WebhookEvent::SubscriptionStatus(s) => { /* ... */ }
        WebhookEvent::SubscriptionPeriodChanged(s) => { /* ... */ }
        WebhookEvent::SubscriptionChange(c) => { /* ... */ }
    }
    // {"message":"success"} (or "failed"), signed with your private key.
    webhook::build_signed_response(client, true)
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
      config.rs     base.rs   crypto.rs   error.rs   common.rs
      biz/          order / refund / subscription / merchant
      webhook/      core + events + notifications + axum integration
  waffo-rs-derive/  # #[derive(WaffoRequest)] proc-macro
```

## Testing

```sh
cargo test
cargo test --features axum
```

Includes RSA sign/verify vectors and JSON wire-format round-trip tests.

## License

Licensed under the [MIT License](LICENSE-MIT).
