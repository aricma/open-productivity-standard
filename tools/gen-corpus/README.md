# gen-corpus

Generates `ops/tests/acceptance/**` from `test-corpus/` — one `#[test]` per
corpus case — so the suite is committed, reviewable, and mirrors the corpus
layout.

## What it does

- Walks `../test-corpus`.
- For `valid/` and `invalid/`, writes one module per input format
  (`valid/json.rs`, `invalid/jsonl.rs`, …) with a `#[test]` per case.
- For `convert/` and `roundtrips/`, writes a single file each, since those
  cases are cross-format.
- Names each test after its case directory, snake_cased. The case already
  carries a `claim.txt`; the generator copies it into the test's failure
  message.
- Emits `#[ignore]` for a case whose format the crate does not implement,
  so skips stay visible and counted.
- Owns `ops/tests/acceptance/`: it writes every file it lists and removes
  any other `.rs` in that directory.

## Running

```text
cargo run -p gen-corpus             regenerate, removing stale files
cargo run -p gen-corpus -- --check  fail if the committed files differ
```

`mask gen-corpus [--check]` wraps both. `mask test` regenerates before it
runs, and `mask ci` is `format`, `build`, and that test run — deliberately
without `--check`, since regeneration would repair a stale tree in the same
run.

## Notes

- The generated tests are committed and reviewed like any other code. Do
  not edit them by hand — change the corpus and regenerate.
- The format name → variant mapping is duplicated here, because this is a
  separate crate and cannot depend on `ops-lib`. Keep it in sync when a
  format is added.
