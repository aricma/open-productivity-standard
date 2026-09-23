# Integration tests

Black-box tests that run against `ops-lib`'s public API, one file per
concern:

- `corpus.rs` — runs the portable corpus in `../../test-corpus`.
- `model.rs` — model-level checks that cannot be expressed as a corpus
  document: in-memory invalid models, writer determinism, and the
  full-model round-trip.
