//! Crypto conformance tests for `waffo_rs::crypto`, benchmarked against the Go
//! SDK (`waffo-go`).
//!
//! Two layers:
//!
//! 1. **Cross-language fixed vectors** — a key pair and signatures emitted by the
//!    *actual* Go SDK (`utils.GenerateKeyPair` + `utils.Sign`). The signature
//!    scheme is RSASSA-PKCS1-v1_5 over SHA-256, which is **deterministic**, so a
//!    correct Rust port must reproduce these exact signature bytes and must
//!    verify the Go-produced signatures. This proves byte-for-byte interop with
//!    the official SDK.
//!
//! 2. **Behavioural parity** — the same sign/verify/validate scenarios the Go
//!    tests cover (`test/vectors/rsa_vector_test.go`, `test/rsa_utils_test.go`):
//!    every payload they sign, plus tamper / wrong-key / corrupted-input /
//!    invalid-key negatives, run here against freshly generated keys.

use waffo_rs::crypto;

// ---------------------------------------------------------------------------
// Layer 1: cross-language fixed vectors produced by the Go SDK.
//
// Generated once via waffo-go `utils.GenerateKeyPair()` + `utils.Sign(...)`.
// Do not regenerate casually: the whole point is that these bytes are frozen
// and the Rust implementation must match them.
// ---------------------------------------------------------------------------

const GO_PRIVATE_KEY: &str = "MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQDCbVzePVzKaiHR5U/My9NI2hUG+/aO1LRfqyNAjQycOxwghcws4at/AhiZbxgVmktTgzoyFjvPi7hT0q+8vTJXGWxRReVPXHy88FbJ6RED6Zyiwfofi+AX6YDeDIaGkD2nYumzQDN7g1DvIDCuRPjNsr5/Q2RMEEybstB2wYvvEzgx5GEZ2VVhxtIia/mrblkCHv2PJagcmg5PhzHWKnwLTX9DqDfpajYVlq40Ph8Zy7jaJOAspSBRncN6XqRUwoXFwW/zbn6TT+l+lbHuyrSHKXWIWUfSrHVcTdm66He+RqDVcQh0tMLRYAenrvaPueV9uRk9yPJj9hhyvNaZUa9FAgMBAAECggEAK88WoOD0uotFiZUA8SVwOzcgi81UVgSpi/D05YOitsU+5jkfs3E6AklHn7L/m0aD+JJWF5kY6wARjZmojX+YCzYoSPvV2pb9aFlDRQWmFtqZt7a7lEYhPWIY7m+mFEYGDmRkQkaSEx+Yqfj33xydb0P6VpSp0dXOXTribA/aZwjpR27vcfkTphkRLqx9/cV2qIBG7mJwa+broEJGW2EOlb1lukKQcksSPzJN9UUYMUXs7/iMPcPwnfnxBuyIJyi8taZ6SJRW0mqtO3I7h5vi5xPq96kv1kzvJZKWFPmpIFLFzxp0i5IBD+A7aS6hNBZL/Ii9ecuMktDjgJQRspoi4QKBgQDEu9RjCiHM9227uZt9KcM6IQECFeY6f2+GyRN+ovYlYe+LO0UnC+RarM84VDDlrqlCflSMaZxEOxV0S6V6K71DCw401766uPzKWp0O53Bv62IxY/xjaeZC2zBoFDr4wCfVShwFBiCfResVmZ1t3T1zHTqkMhUhj4bRNU59AbJ6mQKBgQD8/6d54FiXJeA71502YMY1kp79LayG1PKB/e7ce6B7FoOz7iuCXcPpELzUsWHik4rs1ZcmSMLQ2xPqaTSCNwM3oKiEtzJVHXDb3/i5U1QxCW5vnE7BOebZt9loBRgBovyohx1n4EIg+yoiy+V+dOH/4Drhqduc2a3xNfvpGuARjQKBgECbNko3/5WiA8VMVMWru1MRl5Upv+uiAewPiHlj5tWr7dCZbEJAY4NrkLl83Hnw++C8P+PEou69QHRqizMtvf+QV9/+ocIMEegaDc3hL0lx0VNK9I1pL5bxCFqFmkAqSmp+5ei4dGoZufPj+JElwJJXHPTjBZF54to9WUMwFX6xAoGAYeJX3aObIv+Yfg6x2LQge+G6eaIOOixxlzG0FMIQGQ9g7WCcnNfCUI4VQQVQJeWHvpH05O8J0NyyG+OQWUEHkaIrEQ3//0Zgv/ErvigrTr2jaLNFFSVd16Z2CvTNkPSKZHnCOspdS31hlAznQcHfHqyWm3Coc0sVLVoBmPK49pECgYAeuQD3XBQgJ4BjWByynUApAmzFaYbexwrk7BomPNzxr9p1tYIzLKv8CYpgJPp0MKKeK+n/ZGu4ZPxt9WJkGF/osfirv5DgY7CKPjmBxng5osIuGT18afnu9KD4uiPgnnLqYmTVIlB5DvS/MnhUPuzwljQYenV90PWPqCw828+k7g==";

const GO_PUBLIC_KEY: &str = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwm1c3j1cymoh0eVPzMvTSNoVBvv2jtS0X6sjQI0MnDscIIXMLOGrfwIYmW8YFZpLU4M6MhY7z4u4U9KvvL0yVxlsUUXlT1x8vPBWyekRA+mcosH6H4vgF+mA3gyGhpA9p2Lps0Aze4NQ7yAwrkT4zbK+f0NkTBBMm7LQdsGL7xM4MeRhGdlVYcbSImv5q25ZAh79jyWoHJoOT4cx1ip8C01/Q6g36Wo2FZauND4fGcu42iTgLKUgUZ3Del6kVMKFxcFv825+k0/pfpWx7sq0hyl1iFlH0qx1XE3Zuuh3vkag1XEIdLTC0WAHp672j7nlfbkZPcjyY/YYcrzWmVGvRQIDAQAB";

/// `(message, signature)` pairs the Go SDK produced with [`GO_PRIVATE_KEY`].
/// The Unicode case also pins UTF-8 byte handling across the two languages.
const GO_VECTORS: &[(&str, &str)] = &[
    (
        r#"{"paymentRequestId":"test-123","amount":"100.00"}"#,
        "rEIzE1b1Mn5YzPbQayEKpDRm2srsUqJYeHMMINPwceebBaNpaDpmNgouqkn2EqYviuIuJl4L64gm6eMC9mGAptzBE9A60X9KCqIzV2cZQThYPdj+1LZnL8FdBld5aYvt78lnW9F9LGqq7H0jgA+XjHi9fFv74HWoGYoI/C7SDcjkIu6uGC2VboVFcCAv1WMYC1Jl6zt5mB1iXzf9QEJ2MEYwnmI9Murhvmltwnilms7jOXihl1pjO4Gtp6b1gP6d0acrDn+XVNVzU/zkozzjosOCbVqVPRNQwCntBEKKW1irRFH/0hQoF5ggajY/Myp7+sm1cEUQkJ8sHpmxIchwYA==",
    ),
    (
        r#"{"description":"测试订单","amount":"100.00"}"#,
        "k7BD4mKSiP6eZyQnzWm7b/v5MtgILiIIooy8VKHqrsmyd/rpcazw52tcxJprxe0HH/1Xx4xPsI0NdFY6gTvPrEOQWPmcyXn/E5kypUZNUabHA30OPEqsH4LLjiQP9MF0XptME2gyjbrzGANykhnI18azivUvJ5rcfwAbgai7PJgsDXM2YA2QHW+CF2838pmrm7w0oipQLh3983bUgs9vNZPrFvQlg/sX6Kh6hAIlbUkfDCwIYd9FeJqAU1N5waHwGkoAGAnRzdiXece8pGjSyhF1EJUTvinO7eNPCLpTn8phiaq6jAiklxwtuvjYffbGFEM+WAjt+yrn4QFVBfWKyw==",
    ),
    (
        "",
        "swfHYT2jowUUuwra0uECo9lczv4E4fttSWF1KdMB4L0RmgbqmjD6M2lCW98oKWaEKuBl4O73l+9GNJVN1jYscygQkpX4U+0WBC/+J8ckVYfkvG6JyfZWF1nCGiTm8ZIYjsgUs6pUU5bZGWaw9Nm+lhNdK4VUQOg5AGmwR37sVyVM4qVc4p05ktTFru7B6dNThv+QXefgGuHQOUrdQZIdBNmocUN6Vh3gV5jZ0foNpWQfAuIT6Et/dYCs24zFFkGo0UefLUmYqZiG0p2WpckAki2kpK3+db2Hwpctzoiqs1ItTtXZ1nXqGpGl1KTExI4oYZRqm35q5b1XuC2bowoUqQ==",
    ),
];

#[test]
fn parses_go_generated_keys() {
    crypto::parse_private_key(GO_PRIVATE_KEY)
        .expect("Go-generated PKCS#8 private key should parse");
    crypto::parse_public_key(GO_PUBLIC_KEY).expect("Go-generated X.509 public key should parse");
}

/// The crux of "对标 go SDK": signing the same bytes with the same key must
/// yield the *identical* Base64 signature the Go SDK produced.
#[test]
fn reproduces_go_signatures_byte_for_byte() {
    let private_key = crypto::parse_private_key(GO_PRIVATE_KEY).expect("parse private key");
    for (message, expected) in GO_VECTORS {
        let signature = crypto::sign(&private_key, message.as_bytes()).expect("sign failed");
        assert_eq!(
            &signature, expected,
            "Rust signature differs from the Go SDK for message {message:?}"
        );
    }
}

/// The Rust verifier must accept signatures the Go SDK produced.
#[test]
fn verifies_go_produced_signatures() {
    let public_key = crypto::parse_public_key(GO_PUBLIC_KEY).expect("parse public key");
    for (message, signature) in GO_VECTORS {
        crypto::verify(&public_key, message.as_bytes(), signature)
            .unwrap_or_else(|_| panic!("verify() should accept the Go signature for {message:?}"));
    }
}

/// A Go-produced signature must not verify under an unrelated public key.
#[test]
fn go_signature_rejected_under_wrong_key() {
    let (message, signature) = GO_VECTORS[0];
    let other = crypto::generate_key_pair().expect("generate_key_pair failed");
    let wrong_pub = crypto::parse_public_key(&other.public_key).expect("parse public key");
    assert!(
        crypto::verify(&wrong_pub, message.as_bytes(), signature).is_err(),
        "a Go signature must not verify under a different public key"
    );
}

// ---------------------------------------------------------------------------
// Layer 2: behavioural parity with the Go test suite (generated keys).
// ---------------------------------------------------------------------------

/// The exact payloads the Go `TestSignAndVerify` / `TestRSA00x` cases sign.
const PAYLOADS: &[&str] = &[
    // basic_json
    r#"{"paymentRequestId":"test-123","amount":"100.00"}"#,
    // unicode_data
    r#"{"description":"测试订单","amount":"100.00"}"#,
    // empty_object
    r"{}",
    // complex_nested
    r#"{"order":{"id":"123","items":[{"name":"Product","qty":1}]},"user":{"email":"test@example.com"}}"#,
    // special_characters
    r#"{"desc":"Test & Demo <script>alert('xss')</script>","amount":"99.99"}"#,
    // large_payload (RSA-006)
    r#"{"items":[{"id":1,"name":"Product 1","price":"10.00"},{"id":2,"name":"Product 2","price":"20.00"},{"id":3,"name":"Product 3","price":"30.00"}],"metadata":{"version":"1.0","timestamp":"2024-01-01T00:00:00Z"}}"#,
    // empty_string (RSA-007)
    "",
];

/// Generate a key pair (Base64) and parse both halves — mirrors the Go
/// `getTestKeyPair` helper combined with the SDK's parse step.
fn test_key_pair() -> (rsa::RsaPrivateKey, rsa::RsaPublicKey) {
    let kp = crypto::generate_key_pair().expect("generate_key_pair failed");
    let priv_key = crypto::parse_private_key(&kp.private_key).expect("parse_private_key failed");
    let pub_key = crypto::parse_public_key(&kp.public_key).expect("parse_public_key failed");
    (priv_key, pub_key)
}

// RSA-001..007 / TestSignAndVerify: sign each payload, verify succeeds with the
// matching public key, and verify fails for tampered data (Go appends "x").
#[test]
fn sign_and_verify_roundtrip() {
    let (priv_key, pub_key) = test_key_pair();

    for data in PAYLOADS {
        let signature = crypto::sign(&priv_key, data.as_bytes()).expect("sign failed");
        assert!(!signature.is_empty(), "sign() returned an empty signature");

        crypto::verify(&pub_key, data.as_bytes(), &signature)
            .unwrap_or_else(|_| panic!("verify() should accept a valid signature for: {data:?}"));

        let mut tampered = (*data).to_string();
        tampered.push('x');
        assert!(
            crypto::verify(&pub_key, tampered.as_bytes(), &signature).is_err(),
            "verify() should reject tampered data for: {data:?}"
        );
    }
}

// RSA-V03: isolated original-vs-tampered check.
#[test]
fn verify_tampered_data_fails() {
    let (priv_key, pub_key) = test_key_pair();
    let original = r#"{"test":"original"}"#;
    let tampered = r#"{"test":"tampered"}"#;

    let signature = crypto::sign(&priv_key, original.as_bytes()).expect("sign failed");

    crypto::verify(&pub_key, original.as_bytes(), &signature)
        .expect("verify() should accept the original data");
    assert!(
        crypto::verify(&pub_key, tampered.as_bytes(), &signature).is_err(),
        "verify() should reject tampered data"
    );
}

// RSA-V02 / TestVerifyInvalidSignature: a syntactically valid Base64 string that
// is not a real signature must be rejected.
#[test]
fn verify_invalid_signature_fails() {
    let (_priv_key, pub_key) = test_key_pair();
    let data = r#"{"test":"data"}"#;

    // "invalid-signature" Base64-encoded — valid Base64, bogus signature.
    let invalid_signature = "aW52YWxpZC1zaWduYXR1cmU=";
    assert!(
        crypto::verify(&pub_key, data.as_bytes(), invalid_signature).is_err(),
        "verify() should reject a valid-Base64 but bogus signature"
    );
}

// RSA-E04 / TestVerifyInvalidSignature: corrupted (non-Base64) signature must be
// handled gracefully (returns an error, never panics).
#[test]
fn verify_corrupted_base64_signature_fails() {
    let (_priv_key, pub_key) = test_key_pair();
    let data = r#"{"test":"data"}"#;

    assert!(
        crypto::verify(&pub_key, data.as_bytes(), "!!!invalid-base64!!!").is_err(),
        "verify() should reject a corrupted Base64 signature"
    );
}

// RSA-V04 / TestCrossKeyVerification: a signature must not verify under a
// different key pair's public key.
#[test]
fn verify_wrong_public_key_fails() {
    let (priv1, pub1) = test_key_pair();
    let (_priv2, pub2) = test_key_pair();
    let data = r#"{"test":"cross-key-test"}"#;

    let signature = crypto::sign(&priv1, data.as_bytes()).expect("sign failed");

    crypto::verify(&pub1, data.as_bytes(), &signature)
        .expect("verify() should accept with the matching key pair");
    assert!(
        crypto::verify(&pub2, data.as_bytes(), &signature).is_err(),
        "verify() should reject with a wrong public key"
    );
}

// RSA-E01 / TestValidatePrivateKey: parsing an invalid / empty private key must
// error (the Rust analog of Go's ValidatePrivateKey returning an error).
#[test]
fn parse_invalid_private_key_errors() {
    assert!(
        crypto::parse_private_key("NOT_A_VALID_KEY").is_err(),
        "parse_private_key() should error on an invalid key"
    );
    assert!(
        crypto::parse_private_key("").is_err(),
        "parse_private_key() should error on an empty key"
    );
}

// RSA-E02 / TestValidatePublicKey: parsing an invalid / empty public key must
// error. (Go's `Verify` returns `false` for a bad key; the Rust split surfaces
// the bad key at parse time as an error instead.)
#[test]
fn parse_invalid_public_key_errors() {
    assert!(
        crypto::parse_public_key("NOT_A_VALID_KEY").is_err(),
        "parse_public_key() should error on an invalid key"
    );
    assert!(
        crypto::parse_public_key("").is_err(),
        "parse_public_key() should error on an empty key"
    );
}

// RSA-E01 (sign side): signing with an unparseable key errors rather than
// producing a bogus signature.
#[test]
fn sign_with_invalid_key_errors() {
    assert!(
        crypto::parse_private_key("NOT_A_VALID_KEY").is_err(),
        "an invalid signing key must be rejected before use"
    );
}

// TestGenerateKeyPair / TestKeyPairGeneration: a freshly generated pair is
// non-empty, parses cleanly and round-trips a sign/verify.
#[test]
fn generated_key_pair_is_valid_and_round_trips() {
    let kp = crypto::generate_key_pair().expect("generate_key_pair failed");
    assert!(!kp.private_key.is_empty(), "generated private key is empty");
    assert!(!kp.public_key.is_empty(), "generated public key is empty");

    let priv_key =
        crypto::parse_private_key(&kp.private_key).expect("generated private key should parse");
    let pub_key =
        crypto::parse_public_key(&kp.public_key).expect("generated public key should parse");

    let data = r#"{"test":"keypair-generation"}"#;
    let signature = crypto::sign(&priv_key, data.as_bytes()).expect("sign failed");
    crypto::verify(&pub_key, data.as_bytes(), &signature)
        .expect("verify() should accept a signature from the generated pair");
}
