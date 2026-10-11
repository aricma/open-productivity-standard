# ops-lib

A Rust reader/writer for the
[Open Productivity Standard (OPS)](../specs/v0/model.md).

> **Alpha.** Nothing is released yet. The standard itself is still at
> version `0` and may change. Expect breakage — the API and the data
> format are both in flux. The [changelog](CHANGELOG.md) lists what the
> current interface supports.

## What this is

- **Read** OPS documents (JSON, YAML, JSONL, CSV) into a
  **forest** of tasks — zero or more roots — and **write** it back out,
  losslessly — the round-trip contract is tested against the portable
  corpus, read in place, so the spec and the library cannot drift apart
  unnoticed.
- **Versioned by spec**: the API is namespaced by OPS version
  (`ops_lib::v0`), so the next version lands beside v0 without breaking
  callers that pin the old one.
- **Validates** on both sides: parsing never returns an invalid forest,
  and in-memory models are checked before serializing (duplicate ids,
  metadata key charset, version placement and supported versions, and a
  `done` task with an `open` subtask).
- **Writes the same bytes twice**: output is idempotent — write, read,
  write again is byte-stable — so reformatting a file never churns it.
  Layout matters where a format depends on it (CSV quoting, YAML
  nesting), and the writer emits one stable, minimal form.
- **Bounded**: explicit resource guardrails (`ops_lib::v0::limits`) cap
  document size, task count, nesting depth, and field and metadata sizes,
  so hostile input is rejected with a coded error instead of exhausting
  memory or the stack.

## Repository layout

This crate is one half of the OPS monorepo; the standard lives in
[`../specs/`](../specs/).

```
ops/
├── Cargo.toml           the crate manifest
├── formats.txt          the formats this crate claims, and its Rust path
├── lib/                 the library (source only)
│   ├── lib.rs           crate root: the versioned public surface
│   ├── shared/          helpers shared by every spec version
│   │   └── version.rs   the official release list + version check
│   └── v0/              the version-0 implementation
│       ├── mod.rs       read/write + the published types
│       ├── version.rs   applies the shared release check to v0 tasks
│       ├── doc/         the OpsDoc trait + one file per format
│       │                (json, yaml, jsonl, csv); reads/writes raw_task
│       └── model/       the data model: task, status, flat_task, raw_task
├── tests/               integration tests
│   ├── acceptance/      generated from ../../specs/v0/tests
│   ├── essentials/      hand-written model-level checks
│   ├── security/        guardrail and hostile-input checks
│   └── performance/     the linearity tripwire
├── README.md
├── SECURITY.md
└── CHANGELOG.md
```

## Local development

Every action is defined in the [maskfile](../maskfile.md) at the repo
root; run it from there. To cut a release, see
[`docs/Release.md`](../docs/Release.md).
