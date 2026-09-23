//! Table-driven roundtrip tests: the OPS lossless contract.

mod common;

use common::models::full_model;
use common::run_test_cases::run_write_roundtrip_cases;
use common::test_case::TestCase;
use ops_lib::Format;

/// The full model (every field, two nesting levels) roundtrips
/// losslessly through every supported format — nested tree formats and
/// the flat form.
#[test]
fn full_model_roundtrips_in_every_format() {
    let cases = [
        TestCase {
            message: "full model roundtrips in json".into(),
            given: (Format::Json, full_model()),
            expected: vec![full_model()],
        },
        TestCase {
            message: "full model roundtrips in yaml".into(),
            given: (Format::Yaml, full_model()),
            expected: vec![full_model()],
        },
        TestCase {
            message: "full model roundtrips in jsonl".into(),
            given: (Format::Jsonl, full_model()),
            expected: vec![full_model()],
        },
    ];

    run_write_roundtrip_cases(&cases);
}
