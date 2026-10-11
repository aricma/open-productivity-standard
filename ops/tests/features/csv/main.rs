//! The build that claims `csv`: csv carries a document, and no format the
//! standard documents is a format here unless its own feature is on too.

#[path = "../common/mod.rs"]
mod common;

use ops_lib::v0::Format;

/// csv carries the document there and back.
#[test]
fn csv_round_trips() {
    common::round_trip(Format::Csv);
}

/// Nothing is a format here that this build does not claim.
#[test]
fn formats_are_claimed_exactly_when_their_feature_is_on() {
    common::assert_formats_are_claimed_exactly_when_their_feature_is_on();
}
