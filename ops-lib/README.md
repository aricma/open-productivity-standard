# ops-lib

A Rust reader/writer for the
[Open Productivity Standard (OPS)](../specs/open_productivity_standard_v0.md).

> **Alpha.** Nothing is released yet. The standard itself is still at
> version `0` and may change. Expect breakage — the API and the data
> format are both in flux.

## What this is

- **Read** OPS documents (JSON, YAML, JSONL) into tasks, and **write**
  tasks back out, losslessly — the round-trip contract is tested against
  the standard's own examples and fixtures, read in place, so the spec
  and the library cannot drift apart unnoticed.
- **Validates** on both sides: parsing never returns an invalid forest,
  and in-memory models are checked before serializing (duplicate ids,
  metadata key charset, version placement and released versions).
- The canonical, single-source-of-truth implementation of the standard;
  everything else is a thin wrapper around it.

## Status

- Formats: JSON, YAML, JSONL. CSV and Markdown are specced but not
  implemented yet; their examples and fixtures wait for support (see the
  root `TODOs.md`).
- Conformance: `tests/fixtures/ops/` holds the standard's examples and
  conformance fixtures, and the test suite reads them in place — change
  an example or fixture and CI stops passing.

## Repository layout

This crate is one half of the OPS monorepo; the standard lives in
[`../specs/`](../specs/).

```
ops-lib/
├── src/                 the library
│   ├── lib.rs           facade: read/write + re-exports
│   ├── doc/             the OpsDoc trait, one file per serialization
│   │                    (json, yaml, jsonl) + shared validation helpers
│   └── model/           the data model: task, status, flat_task, error
├── tests/               integration suite (see tests/README.md)
│   ├── common/          shared test scaffolding
│   └── fixtures/        ops/ (the standard's documents) + local/ (ours)
└── Cargo.toml
```

## Local development

The [maskfile](../maskfile.md) at the repo root documents every action —
run it from the root, where `mask ci` mirrors the CI pipeline:

```sh
mask ci              # full pipeline (format + build + test)
mask format          # cargo fmt --check
mask build           # check + clippy + docs + audit + deny
mask test            # cargo test
```

## License

MIT (see the root [`LICENSE`](../LICENSE)).
