# gen-corpus

Generates `ops/tests/acceptance/**` from `specs/<version>/tests/` — one
`#[test]` per corpus case — so the suite is committed, reviewable, and
mirrors the corpus layout.

## What it does

- Walks every `specs/<version>/tests/`, and writes that version's cases to
  `ops/tests/acceptance/<version>/`.
- Groups by format first, then by kind: `<format>/valid/`,
  `<format>/invalid/`, `<format>/roundtrip/`, and the cross-format
  `transform/`. Each becomes a module. An `invalid` case asserts on read or
  on write, depending on whether its `given` is in the enclosing format.
- Names each test after its case directory, snake_cased. The case already
  carries a `claim.txt`; the generator copies it into the test's failure
  message.
- Emits `#[ignore]` for a case whose format the crate does not claim, so
  skips stay visible and counted.
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
- Which formats are supported comes from `ops-lib`'s `Format` enum, so the
  generator and the crate cannot disagree: a format `ops-lib` stops
  claiming turns into `#[ignore]`d tests on the next run.
