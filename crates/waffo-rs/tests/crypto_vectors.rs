//! RSA signing / verification cross-check, ported from the Go vector tests:
//!   waffo-go/test/vectors/rsa_vector_test.go
//!   waffo-go/test/rsa_utils_test.go
//!
//! The Go vectors use *generated* key pairs (the checked-in
//! test-vectors/rsa-signing.json holds placeholder keys), so there is no fixed
//! signature to pin. We therefore replicate the intent: a
//! generate -> parse -> sign -> verify roundtrip over the exact same payloads
//! the Go test signs, plus tamper / wrong-key / corrupted-input negative checks
//! and a parse-known-key check.
//!
//! Scheme (identical to the Go SDK): RSASSA-PKCS1-v1_5 over SHA-256, Base64.

use waffo_rs::crypto;

/// The exact payloads the Go `TestSignAndVerify` / `TestRSA00x` cases sign.
/// Copied verbatim (including the Unicode and special-character cases).
const PAYLOADS: &[&str] = &[
    // basic_json
    r#"{"paymentRequestId":"test-123","amount":"100.00"}"#,
    // unicode_data
    r#"{"description":"测试订单","amount":"100.00"}"#,
    // empty_object
    r#"{}"#,
    // complex_nested
    r#"{"order":{"id":"123","items":[{"name":"Product","qty":1}]},"user":{"email":"test@example.com"}}"#,
    // special_characters
    r#"{"desc":"Test & Demo <script>alert('xss')</script>","amount":"99.99"}"#,
    // large_payload (RSA-006)
    r#"{"items":[{"id":1,"name":"Product 1","price":"10.00"},{"id":2,"name":"Product 2","price":"20.00"},{"id":3,"name":"Product 3","price":"30.00"}],"metadata":{"version":"1.0","timestamp":"2024-01-01T00:00:00Z"}}"#,
    // empty_string (RSA-007)
    "",
];

/// Generate a key pair (Base64) and parse both halves, mirroring the Go
/// `getTestKeyPair` helper combined with the SDK's parse step.
fn test_key_pair() -> (rsa::RsaPrivateKey, rsa::RsaPublicKey) {
    let kp = crypto::generate_key_pair().expect("generate_key_pair failed");
    let priv_key = crypto::parse_private_key(&kp.private_key).expect("parse_private_key failed");
    let pub_key = crypto::parse_public_key(&kp.public_key).expect("parse_public_key failed");
    (priv_key, pub_key)
}

// RSA-001..007 / TestSignAndVerify: sign each payload, verify succeeds with the
// matching public key, and verify fails for tampered data.
#[test]
fn sign_and_verify_roundtrip() {
    let (priv_key, pub_key) = test_key_pair();

    for data in PAYLOADS {
        let signature = crypto::sign(&priv_key, data.as_bytes()).expect("sign failed");
        assert!(!signature.is_empty(), "sign() returned an empty signature");

        // Verify with the correct public key succeeds.
        crypto::verify(&pub_key, data.as_bytes(), &signature)
            .unwrap_or_else(|_| panic!("verify() should accept a valid signature for: {data:?}"));

        // Verify with tampered data fails (Go appends "x").
        let mut tampered = data.to_string();
        tampered.push('x');
        assert!(
            crypto::verify(&pub_key, tampered.as_bytes(), &signature).is_err(),
            "verify() should reject tampered data for: {data:?}"
        );
    }
}

// RSA-V03 / TestSignAndVerify tamper case, isolated with explicit payloads.
#[test]
fn verify_tampered_data_fails() {
    let (priv_key, pub_key) = test_key_pair();
    let original = r#"{"test":"original"}"#;
    let tampered = r#"{"test":"tampered"}"#;

    let signature = crypto::sign(&priv_key, original.as_bytes()).expect("sign failed");

    // Original verifies.
    crypto::verify(&pub_key, original.as_bytes(), &signature)
        .expect("verify() should accept the original data");

    // Tampered does not.
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

    // Matching key verifies.
    crypto::verify(&pub1, data.as_bytes(), &signature)
        .expect("verify() should accept with the matching key pair");

    // Wrong key does not.
    assert!(
        crypto::verify(&pub2, data.as_bytes(), &signature).is_err(),
        "verify() should reject with a wrong public key"
    );
}

// RSA-E01 / TestValidatePrivateKey: parsing an obviously invalid private key
// must error (it is not valid Base64 / not a key).
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

// RSA-E02 / TestValidatePublicKey: parsing an obviously invalid public key must
// error.
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

// TestGenerateKeyPair / TestKeyPairGeneration: a freshly generated pair parses
// cleanly and round-trips a sign/verify.
#[test]
fn generated_key_pair_is_valid_and_round_trips() {
    let kp = crypto::generate_key_pair().expect("generate_key_pair failed");
    assert!(!kp.private_key.is_empty(), "generated private key is empty");
    assert!(!kp.public_key.is_empty(), "generated public key is empty");

    // Both halves parse (cross-checks the parse-known-key path against keys the
    // SDK itself produced, in PKCS#8 / X.509 SPKI Base64 form).
    let priv_key = crypto::parse_private_key(&kp.private_key)
        .expect("generated private key should parse");
    let pub_key =
        crypto::parse_public_key(&kp.public_key).expect("generated public key should parse");

    let data = r#"{"test":"keypair-generation"}"#;
    let signature = crypto::sign(&priv_key, data.as_bytes()).expect("sign failed");
    crypto::verify(&pub_key, data.as_bytes(), &signature)
        .expect("verify() should accept a signature from the generated pair");
}
