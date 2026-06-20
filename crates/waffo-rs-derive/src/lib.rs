//! Derive macros for `waffo-rs`.
//!
//! `#[derive(WaffoRequest)]` generates a `WaffoRequest` impl that performs the
//! pre-send field injection (merchant id / requested-at timestamp) the Go SDK
//! does via reflection. Fields opt in with `#[waffo(...)]`:
//!
//! - `#[waffo(requested_at)]`  — `Option<String>`; filled with the current
//!   ISO-8601 timestamp when empty.
//! - `#[waffo(merchant_id)]`   — flat `Option<String>`; filled with the
//!   configured merchant id when empty.
//! - `#[waffo(merchant_info)]` — nested `Option<M>` where `M: Default +
//!   MerchantInfoExt`; the merchant id is set on the nested struct when empty.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(WaffoRequest, attributes(waffo))]
pub fn derive_waffo_request(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_g, ty_g, where_g) = input.generics.split_for_impl();

    let mut stmts = Vec::new();

    if let Data::Struct(s) = &input.data {
        if let Fields::Named(named) = &s.fields {
            for field in &named.named {
                let ident = field.ident.as_ref().unwrap();
                let mut kind: Option<&'static str> = None;

                for attr in &field.attrs {
                    if !attr.path().is_ident("waffo") {
                        continue;
                    }
                    let res = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("requested_at") {
                            kind = Some("requested_at");
                        } else if meta.path.is_ident("merchant_id") {
                            kind = Some("merchant_id");
                        } else if meta.path.is_ident("merchant_info") {
                            kind = Some("merchant_info");
                        } else {
                            return Err(meta.error("unknown `waffo` attribute"));
                        }
                        Ok(())
                    });
                    if let Err(e) = res {
                        return e.to_compile_error().into();
                    }
                }

                match kind {
                    Some("requested_at") => stmts.push(quote! {
                        if self.#ident.as_deref().unwrap_or("").is_empty() {
                            self.#ident = ::core::option::Option::Some(__ctx.now.to_string());
                        }
                    }),
                    Some("merchant_id") => stmts.push(quote! {
                        if let ::core::option::Option::Some(__mid) = __ctx.merchant_id {
                            if self.#ident.as_deref().unwrap_or("").is_empty() {
                                self.#ident = ::core::option::Option::Some(__mid.to_string());
                            }
                        }
                    }),
                    Some("merchant_info") => stmts.push(quote! {
                        if let ::core::option::Option::Some(__mid) = __ctx.merchant_id {
                            ::waffo_rs::base::MerchantInfoExt::set_merchant_id_if_empty(
                                self.#ident.get_or_insert_with(::core::default::Default::default),
                                __mid,
                            );
                        }
                    }),
                    _ => {}
                }
            }
        }
    }

    let expanded = quote! {
        impl #impl_g ::waffo_rs::base::WaffoRequest for #name #ty_g #where_g {
            fn inject(&mut self, __ctx: &::waffo_rs::base::InjectCtx<'_>) {
                #(#stmts)*
            }
        }
    };

    expanded.into()
}
