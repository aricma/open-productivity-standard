//! Boundary and security tests: the guardrails in `ops_lib::v0::limits`
//! and hostile input shapes. Every size derives from a constant, so
//! raising one moves the test with it.

use ops_lib::v0::limits::*;
use ops_lib::v0::{self, Format};

/// The document must be rejected with this code.
fn rejected(input: &str, format: Format, code: &str) {
    let err = v0::read(format, input).unwrap_err();
    assert_eq!(
        err.code(),
        code,
        "expected {code} for {} bytes",
        input.len()
    );
}

#[test]
fn document_size_guard() {
    let base = r#"{"title":"t","status":"open"}"#;
    let mut doc = String::with_capacity(MAX_INPUT_BYTES);
    doc.push_str(base);
    doc.push_str(&" ".repeat(MAX_INPUT_BYTES - base.len()));
    assert_eq!(doc.len(), MAX_INPUT_BYTES);
    assert!(v0::read(Format::Json, &doc).is_ok());

    doc.push(' ');
    rejected(&doc, Format::Json, "input-too-large");
}
