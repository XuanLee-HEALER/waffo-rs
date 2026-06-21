//! Smoke-covers every business endpoint free function against a catch-all mock,
//! exercising each `Endpoint` declaration + the uniform `send` path end to end.

use waffo_rs::biz::{merchant, order, refund, subscription};
use waffo_rs::{Client, WaffoConfig, crypto};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_for(base_url: &str) -> Client {
    let merchant = crypto::generate_key_pair().unwrap();
    let waffo = crypto::generate_key_pair().unwrap();
    let cfg = WaffoConfig::builder()
        .api_key("test-key")
        .private_key(&merchant.private_key)
        .waffo_public_key(&waffo.public_key)
        .merchant_id("M-TEST")
        .base_url(base_url)
        .build()
        .unwrap();
    Client::new(cfg).unwrap()
}

#[tokio::test]
async fn every_endpoint_round_trips() {
    let server = MockServer::start().await;
    // Catch-all: any POST returns an empty success envelope, which decodes into
    // every endpoint's response data type.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"code":"0","data":{}}"#))
        .mount(&server)
        .await;
    let client = client_for(&server.uri());

    // order
    order::create(&client, order::CreateOrderParams::default(), None)
        .await
        .unwrap();
    order::inquiry(&client, order::InquiryOrderParams::default(), None)
        .await
        .unwrap();
    order::cancel(&client, order::CancelOrderParams::default(), None)
        .await
        .unwrap();
    order::refund(&client, order::RefundOrderParams::default(), None)
        .await
        .unwrap();
    order::capture(&client, order::CaptureOrderParams::default(), None)
        .await
        .unwrap();

    // refund
    refund::inquiry(&client, refund::InquiryRefundParams::default(), None)
        .await
        .unwrap();

    // subscription
    subscription::create(
        &client,
        subscription::CreateSubscriptionParams::default(),
        None,
    )
    .await
    .unwrap();
    subscription::inquiry(
        &client,
        subscription::InquirySubscriptionParams::default(),
        None,
    )
    .await
    .unwrap();
    subscription::cancel(
        &client,
        subscription::CancelSubscriptionParams::default(),
        None,
    )
    .await
    .unwrap();
    subscription::manage(
        &client,
        subscription::ManageSubscriptionParams::default(),
        None,
    )
    .await
    .unwrap();
    subscription::change(
        &client,
        subscription::ChangeSubscriptionParams::default(),
        None,
    )
    .await
    .unwrap();
    subscription::change_inquiry(&client, subscription::ChangeInquiryParams::default(), None)
        .await
        .unwrap();
    subscription::update(
        &client,
        subscription::UpdateSubscriptionParams::default(),
        None,
    )
    .await
    .unwrap();

    // merchant / pay-method config
    merchant::merchant_config_inquiry(
        &client,
        merchant::InquiryMerchantConfigParams::default(),
        None,
    )
    .await
    .unwrap();
    merchant::pay_method_config_inquiry(
        &client,
        merchant::InquiryPayMethodConfigParams::default(),
        None,
    )
    .await
    .unwrap();
}
