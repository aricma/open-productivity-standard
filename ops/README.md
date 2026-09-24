# ops-lib

A Rust reader/writer for the
[Open Productivity Standard (OPS)](../specs/open_productivity_standard_v0.md).

> **Alpha.** Nothing is released yet. The standard itself is still at
> version `0` and may change. Expect breakage — the API and the data
> format are both in flux. The [changelog](CHANGELOG.md) lists what the
> current interface supports.

## What this is

- **Read** OPS documents (JSON, YAML, JSONL) into tasks, and **write**
  tasks back out, losslessly — the round-trip contract is tested against
  the portable corpus, read in place, so the spec and the library cannot
  drift apart unnoticed.
- **Versioned by spec**: the API is namespaced by OPS version
  (`ops_lib::v0`), so the next version lands beside v0 without breaking
  callers that pin the old one.
- **Validates** on both sides: parsing never returns an invalid forest,
  and in-memory models are checked before serializing (duplicate ids,
  metadata key charset, version placement and released versions).

## Repository layout

This crate is one half of the OPS monorepo; the standard lives in
[`../specs/`](../specs/).

```
ops/
├── Cargo.toml           the crate manifest
├── lib/                 the library (source only)
│   ├── lib.rs           crate root: the versioned public surface
│   ├── shared/          helpers shared by every spec version
│   │   └── version.rs   the official release list + version check
│   └── v0/              the version-0 implementation
│       ├── mod.rs       read/write + the published types
│       ├── version.rs   applies the shared release check to v0 tasks
│       ├── doc/         the OpsDoc trait + one file per serialization
│       │                (json, yaml, jsonl); reads/writes raw_task
│       └── model/       the data model: task, status, flat_task, raw_task
├── tests/               integration tests
│   ├── corpus.rs        the runner over ../test-corpus
│   └── model.rs         model-level checks not expressible as corpus files
├── README.md
├── SECURITY.md
└── CHANGELOG.md
```

## Local development

Every action is defined in the [maskfile](../maskfile.md) at the repo
root; run it from there. To cut a release, see
[`docs/Release.md`](../docs/Release.md).
