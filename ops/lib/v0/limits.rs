//! Implementation guardrails: the largest document `ops-lib` will build.
//!
//! These are **ops-lib policy, not part of the OPS standard**. Another
//! implementation may choose different numbers, and a document over one
//! of these is rejected here, not declared non-conformant. Every constant
//! documents why its value was chosen and has a boundary test in
//! `tests/security/main.rs`.

/// The whole document, checked before parsing. The parser allocates
/// intermediates proportional to its input, so this is the only bound on
/// total parse memory; 16 MiB is roughly 10x the largest performance
/// case, so real documents never approach it.
pub const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;

/// Tasks per document (tree nodes, or flat records). 10x the largest
/// performance scale, so the suite sits well inside it and can grow an
/// order of magnitude; bounds the flat path's id/child maps.
pub const MAX_TASKS: usize = 100_000;

/// Task nesting, root = 0. Below the recursion budget of the tree walks
/// and serde's serializer, and above the ~64 levels the tree parsers
/// already accept, so it never contradicts an accepted tree document.
/// Measured safe at depth 512 on a 1 MiB thread, where 768 overflows, so
/// 256 leaves a 3x margin.
pub const MAX_DEPTH: usize = 256;

/// Subtasks on one task. `MAX_TASKS / 10`: one node with more direct
/// children than a tenth of the document's task budget is pathological,
/// and this reports the shape precisely.
pub const MAX_CHILDREN: usize = 10_000;

/// `title`, in UTF-8 bytes. A title renders on one line; 1 KiB is about
/// ten long sentences, and keeping it small stops 100 000 tasks from
/// meaning 100 000 titles of any size.
pub const MAX_TITLE_BYTES: usize = 1_024;

/// `notes`, in UTF-8 bytes. Free text legitimately large, so generous;
/// equals `MAX_METADATA_VALUE_BYTES` so every "inline blob" field shares
/// one number.
pub const MAX_NOTES_BYTES: usize = 65_536;

/// `id`, in UTF-8 bytes. Ids are `HashMap` keys and flat references; 256
/// is far beyond any practical slug and keeps hashing cheap.
pub const MAX_ID_BYTES: usize = 256;

/// `version`, in UTF-8 bytes. Fits `MAJOR.MINOR.PATCH`; the value is then
/// checked against the released list, so this is only an early bound.
pub const MAX_VERSION_BYTES: usize = 16;

/// Metadata entries on one task. Generous for tags/links/ids, and bounds
/// per-task map overhead plus regex work.
pub const MAX_METADATA_PAIRS: usize = 64;

/// Metadata key, in UTF-8 bytes. Keys are `^[a-z0-9_]{3,}$`; 64 is about
/// four times the longest sensible key and keeps the regex linear.
pub const MAX_METADATA_KEY_BYTES: usize = 64;

/// One metadata value, as its serialized JSON size in bytes. Values may
/// embed base64 attachments; 64 KiB is the "small inline blob" ceiling
/// and matches `MAX_NOTES_BYTES`.
pub const MAX_METADATA_VALUE_BYTES: usize = 65_536;

/// A guardrail, for a coded rejection. Each limit is one constant above.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    InputBytes,
    Tasks,
    Depth,
    Children,
    Title,
    Notes,
    Id,
    Version,
    MetadataPairs,
    MetadataKey,
    MetadataValue,
}

impl Limit {
    /// The largest value this guardrail accepts.
    pub const fn max(self) -> usize {
        match self {
            Limit::InputBytes => MAX_INPUT_BYTES,
            Limit::Tasks => MAX_TASKS,
            Limit::Depth => MAX_DEPTH,
            Limit::Children => MAX_CHILDREN,
            Limit::Title => MAX_TITLE_BYTES,
            Limit::Notes => MAX_NOTES_BYTES,
            Limit::Id => MAX_ID_BYTES,
            Limit::Version => MAX_VERSION_BYTES,
            Limit::MetadataPairs => MAX_METADATA_PAIRS,
            Limit::MetadataKey => MAX_METADATA_KEY_BYTES,
            Limit::MetadataValue => MAX_METADATA_VALUE_BYTES,
        }
    }

    /// What the limit applies to, for the error message.
    pub const fn name(self) -> &'static str {
        match self {
            Limit::InputBytes => "the document",
            Limit::Tasks => "the task count",
            Limit::Depth => "the nesting depth",
            Limit::Children => "the child count",
            Limit::Title => "title",
            Limit::Notes => "notes",
            Limit::Id => "id",
            Limit::Version => "version",
            Limit::MetadataPairs => "the metadata pair count",
            Limit::MetadataKey => "a metadata key",
            Limit::MetadataValue => "a metadata value",
        }
    }

    /// The unit `got` is measured in, for the error message.
    pub const fn unit(self) -> &'static str {
        match self {
            Limit::InputBytes
            | Limit::Title
            | Limit::Notes
            | Limit::Id
            | Limit::Version
            | Limit::MetadataKey
            | Limit::MetadataValue => "bytes",
            Limit::Tasks | Limit::MetadataPairs => "entries",
            Limit::Depth => "levels",
            Limit::Children => "children",
        }
    }

    /// The library's stable code for this guardrail.
    pub const fn code(self) -> &'static str {
        match self {
            Limit::InputBytes => "input-too-large",
            Limit::Tasks => "too-many-tasks",
            Limit::Depth => "nesting-too-deep",
            Limit::Children => "too-many-children",
            Limit::Title | Limit::Notes | Limit::Id | Limit::Version => "field-too-long",
            Limit::MetadataPairs => "too-many-metadata-pairs",
            Limit::MetadataKey => "metadata-key-too-long",
            Limit::MetadataValue => "metadata-value-too-large",
        }
    }
}

/// Rejects `got` when it is over `limit`.
pub(crate) fn check(limit: Limit, got: usize) -> Result<(), crate::v0::error::ValidationError> {
    if got > limit.max() {
        Err(crate::v0::error::ValidationError::LimitExceeded { limit, got })
    } else {
        Ok(())
    }
}

/// The serialized size of a metadata value, without allocating a copy.
pub(crate) fn metadata_value_bytes(value: &serde_json::Value) -> usize {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0 += buf.len();
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let mut counter = Counter(0);
    match serde_json::to_writer(&mut counter, value) {
        Ok(()) => counter.0,
        Err(_) => usize::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_report_their_constant() {
        assert_eq!(Limit::InputBytes.max(), MAX_INPUT_BYTES);
        assert_eq!(Limit::Tasks.max(), MAX_TASKS);
        assert_eq!(Limit::Depth.max(), MAX_DEPTH);
        assert_eq!(Limit::Children.max(), MAX_CHILDREN);
        assert_eq!(Limit::Title.max(), MAX_TITLE_BYTES);
        assert_eq!(Limit::Notes.max(), MAX_NOTES_BYTES);
        assert_eq!(Limit::Id.max(), MAX_ID_BYTES);
        assert_eq!(Limit::Version.max(), MAX_VERSION_BYTES);
        assert_eq!(Limit::MetadataPairs.max(), MAX_METADATA_PAIRS);
        assert_eq!(Limit::MetadataKey.max(), MAX_METADATA_KEY_BYTES);
        assert_eq!(Limit::MetadataValue.max(), MAX_METADATA_VALUE_BYTES);
    }

    #[test]
    fn limits_report_stable_codes() {
        assert_eq!(Limit::InputBytes.code(), "input-too-large");
        assert_eq!(Limit::Tasks.code(), "too-many-tasks");
        assert_eq!(Limit::Depth.code(), "nesting-too-deep");
        assert_eq!(Limit::Children.code(), "too-many-children");
        assert_eq!(Limit::Title.code(), "field-too-long");
        assert_eq!(Limit::Notes.code(), "field-too-long");
        assert_eq!(Limit::Id.code(), "field-too-long");
        assert_eq!(Limit::Version.code(), "field-too-long");
        assert_eq!(Limit::MetadataPairs.code(), "too-many-metadata-pairs");
        assert_eq!(Limit::MetadataKey.code(), "metadata-key-too-long");
        assert_eq!(Limit::MetadataValue.code(), "metadata-value-too-large");
    }

    #[test]
    fn metadata_value_bytes_counts_serialized_size() {
        assert_eq!(metadata_value_bytes(&serde_json::json!("abc")), 5);
        assert_eq!(metadata_value_bytes(&serde_json::json!(3)), 1);
        assert_eq!(metadata_value_bytes(&serde_json::json!([1, 2])), 5);
    }
}
