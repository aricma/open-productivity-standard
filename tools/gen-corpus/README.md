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
- Which formats are claimed is read from `ops/formats.txt`, the crate's own
  declaration, rather than imported from it: a tool about the standard's corpus
  should not depend on one implementation of it. The library's own tests check
  that the declaration matches `Format` exactly, so the two cannot disagree —
  a format added to one and not the other fails the gate.
