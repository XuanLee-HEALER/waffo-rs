# waffo-rs

[English](README.md) · [中文](README.zh-CN.md)

![coverage](https://img.shields.io/badge/coverage-~95%25-brightgreen)
![rust](https://img.shields.io/badge/rust-1.96%2B-orange)
![license](https://img.shields.io/badge/license-MIT-blue)

[Waffo](https://waffo.com) 支付平台的异步 Rust SDK —— 覆盖订单、退款、订阅、拒付、
商户配置与 webhook,自动完成请求签名与响应/webhook 验签。

本库是官方 Go SDK(`waffo-go`)的 Rust port。它**不是 1:1 翻译**:遵循相同的
wire 协议与领域模型,但按 Rust 异步生态的习惯重新塑形,力求地道、好用。

> **状态:v1.0。** 全部接口已实现、文档齐全并经过测试 —— 单元测试 + `wiremock`
> 传输层测试,以及对真实 sandbox 的端到端测试(覆盖每个接口,以及通过真实隧道
> 验证的全部 6 种 webhook 事件)。暂未发布到 crates.io(目前请用 git 依赖)。

## 特性

- **纯异步**,基于 [`reqwest`](https://docs.rs/reqwest) /
  [`tokio`](https://tokio.rs)。
- **统一的请求管线。** 每个接口只是一个小小的 `Endpoint` 声明(请求/响应类型 +
  路径);单一泛型 `send` 对所有接口执行 注入 → 序列化 → 签名 → 发送 → 验签 →
  拆信封 → 错误映射 的固定流程。
- **编译期字段注入**(`merchantId`、`requestedAt`):通过
  `#[derive(WaffoRequest)]` 过程宏完成,取代 Go SDK 的反射。
- **强类型逐字对齐 Go SDK**(JSON tag),并保留逃生舱:请求侧的 `extraParams`
  与响应侧的 `#[serde(flatten)]` 兜底字段,保证服务端新增字段永不破坏反序列化。
- **单一 `WaffoError`**(`Result<T, WaffoError>`)。服务端 `code != "0"` 即业务
  错误;`E0001`(以及读类方法的传输失败)归为 `UnknownStatus` —— 提示你去
  **重新查询**而不是臆断失败。
- **无注册表的 webhook。** 把原始 body 验签 + 解析成一个 `WebhookEvent` 枚举供你
  `match`,再回一个三态的签名应答(`WebhookAck::{Success, Failed, Unknown}` ——
  `Failed`/`Unknown` 会让 Waffo 重推,最长 24 小时)。可选的轻量 `axum` 集成在
  feature 后面。

## 环境要求

- Rust 1.96+(edition 2024)
- 一个 Tokio runtime(SDK 自身不内置 runtime)

## 安装

暂未发布到 crates.io —— 用 git 依赖:

```toml
[dependencies]
waffo-rs = { git = "https://github.com/XuanLee-HEALER/waffo-rs" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

启用可选的 `axum` 集成:

```toml
waffo-rs = { git = "https://github.com/XuanLee-HEALER/waffo-rs", features = ["axum"] }
```

## 快速上手

### 构造 client

```rust
use waffo_rs::{Client, Environment, WaffoConfig};

let config = WaffoConfig::builder()
    .api_key("YOUR_API_KEY")
    .private_key("BASE64_PKCS8_PRIVATE_KEY")     // 商户私钥
    .waffo_public_key("BASE64_X509_PUBLIC_KEY")  // Waffo 的公钥
    .environment(Environment::Sandbox)
    .merchant_id("YOUR_MERCHANT_ID")             // 自动注入到请求里
    .build()?;

let client = Client::new(config)?;               // 密钥在这里一次性解析
# Ok::<(), waffo_rs::WaffoError>(())
```

`WaffoConfig::from_env()` 读取 `WAFFO_MERCHANT_API_KEY`、`WAFFO_MERCHANT_PRIVATE_KEY`、
`WAFFO_PUBLIC_KEY`、`WAFFO_ENVIRONMENT`、`WAFFO_MERCHANT_ID`。

### 创建订单

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
        pay_method_name: Some("CC_VISA".into()),  // 必须在商户合约内
        ..Default::default()
    }),
    ..Default::default()
};

let data = order::create(&client, params, None).await?;
println!("跳转收银台: {}", data.fetch_redirect_url());
# Ok::<(), waffo_rs::WaffoError>(())
```

其它资源形状一致:
`order::{inquiry, cancel, refund, capture}`、`refund::inquiry`、
`subscription::{create, inquiry, cancel, manage, change, change_inquiry, update}`、
`chargeback::{inquiry, update, accept, list}`、
`merchant::{merchant_config_inquiry, pay_method_config_inquiry}`。每个都是
`fn(&Client, Params, Option<&RequestOptions>) -> Result<Data>`。

### 处理 webhook

务必对**原始请求字节**验签(绝不能用重新序列化后的 body),`match` 事件,再回一个
三态签名应答。入站签名在 `X-SIGNATURE` 请求头里,你的签名应答也走同一个头。

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
        // 验签失败 / 暂时处理不了:别 ack —— Waffo 会重推。
        Err(_)                                          => WebhookAck::Failed,
    };
    // 一律 HTTP 200;body({"message":"success"|"failed"|"unknown"})用你的私钥
    // 签名,Waffo 据此决定是否重推。
    webhook::build_signed_response(client, ack)
}
```

启用 `axum` feature 后,`waffo_rs::webhook::axum` 提供 `signature_from_headers`、
`parse_request`、`signed_response` 三个轻量胶水函数 —— 不含 router,也没有 handler
注册表。

## 错误

所有调用返回 `Result<T, WaffoError>`,其中:

- `WaffoError::Api { code, message }` —— 服务端返回 `code != "0"`。
- `WaffoError::UnknownStatus { .. }` —— 状态不确定(`E0001`,或读/幂等类调用传输
  失败)。**请重新查询,不要臆断失败**;用 `err.is_unknown_status()` 判断。

## 工程结构

```
crates/
  waffo-rs/         # SDK 本体(lib 名:waffo_rs)
    src/
      config.rs   base.rs   crypto.rs
      common/     error + trace + null 容忍反序列化辅助
      biz/        order / refund / subscription / chargeback / merchant
      webhook/    核心 + events + notifications + axum 集成
  waffo-rs-derive/  # #[derive(WaffoRequest)] 过程宏
```

## 测试

```sh
cargo test                  # 单元 + wiremock 传输层测试
cargo test --features axum  # + axum webhook 集成
```

对真实 sandbox 的端到端测试是 `#[ignore]` 的(需要 `.env` 凭证 + 真实网络),用
`cargo test --test e2e -- --ignored` 运行。RSA 签名/验签与 Go SDK 的向量逐字节对齐。

覆盖率门槛 **行 ≥80%**,由 [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov)
把关(`just cov`,HTML 报告在 `target/llvm-cov/`);当前行覆盖率约 **95%**。

## 许可

基于 [MIT License](LICENSE-MIT)。
