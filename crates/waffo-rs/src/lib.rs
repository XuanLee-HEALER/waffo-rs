//! # waffo-rs
//!
//! Async Rust SDK for the Waffo payment platform. Helps web applications
//! integrate Waffo (orders, refunds, subscriptions, configuration, webhooks)
//! with automatic request signing, response/webhook signature verification and
//! strongly-typed domain models.
//!
//! The crate exposes endpoints as free functions taking a [`Client`], e.g.
//! `waffo_rs::biz::order::create(&client, params, None).await?`. All endpoints
//! flow through a single uniform processing path ([`base::send`]).

// Allow the derive macro (and generated code) to refer to this crate as
// `::waffo_rs` even from within the crate itself (mirrors serde's pattern).
extern crate self as waffo_rs;

pub mod base;
pub mod biz;
pub mod common;
pub mod config;
pub mod crypto;
pub mod error;
pub mod webhook;

pub use base::{Client, Endpoint, ExtraParams, WaffoRequest};
pub use config::{ConfigBuilder, Environment, RequestOptions, WaffoConfig};
pub use crypto::{generate_key_pair, KeyPair};
pub use error::{Result, WaffoError};

// Derive macro; shares its name with the `WaffoRequest` trait (like serde's
// `Serialize` trait + derive). Lives in a separate proc-macro crate.
pub use waffo_rs_derive::WaffoRequest;
