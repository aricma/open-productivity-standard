# Corpus test code generation — design

Date: 2026-09-24
Status: approved

## Goal

Make every `test-corpus/` case its own named, self-describing test in
`cargo test --test corpus`, instead of three loop-based tests that bury
individual cases.

## Approach

Generate tests at build time via `ops/build.rs`. A `macro_rules!` macro
cannot read the filesystem, so enumeration happens in a build script; the
generated source is `include!`d from the test file.

## Components

### `ops/build.rs` (new)

Walks `../test-corpus` recursively (mirroring the existing `collect()`).
For each case directory it:

- parses `claim.txt`, `given.<ext>`, and optional `expectation.<ext>`;
- derives the group (`valid` / `invalid` / `convert` / `roundtrips`) from
  the first path component;
- derives the input format from `given`'s extension and, where present, the
  output format from `expectation`'s extension via `Format::from_name`;
- determines support: `given` format known, and expectation format known
  when one is present.

It asserts at build time that `convert/` and `roundtrips/` cases carry an
`expectation.<ext>` (moved from test time).

For every case it builds a test name:

```
<group>__<path>__<claim_slug>
```

- `<group>` — `valid`, `invalid`, `convert`, or `roundtrips`.
- `<path>` — the case directory path under the group, `/` → `_`.
- `<claim_slug>` — `claim.txt` trimmed, lowercased, runs of
  non-alphanumeric characters collapsed to a single `_`, then truncated to
  the first 120 characters of the result.

Name collisions (two paths + claims sanitizing to the same identifier) get
a numeric suffix (`_2`, `_3`, …). Names must be valid Rust identifiers.

It writes the generated source to `OUT_DIR/corpus_cases.rs`:

- supported case → `#[test] fn <name>() { … }`;
- unsupported case → `#[test] #[ignore = "unsupported format: <fmt>"] fn <name>() { … }`,
  so skips remain visible in `cargo test` output and are counted, per the
  corpus README.

Bodies replicate today's assertions:

- `valid/` — `ops_lib::v0::read(given_format, &given)` must succeed.
- `invalid/` — the same read must fail.
- `convert/` / `roundtrips/` — read `given`, write with the expectation's
  format, read the result back, read the expectation, and assert the two
  models are equal.

Panic messages keep the full case path and claim.

Emits `cargo:rerun-if-changed=../test-corpus` so adding or editing a case
regenerates the tests.

### `ops/tests/corpus.rs`

Reduced to a module doc comment and:

```rust
include!(concat!(env!("OUT_DIR"), "/corpus_cases.rs"));
```

The runtime `collect()`, `corpus()`, `Case`, `Group`, and the three
loop-based `#[test]` functions are deleted.

## Unchanged

- `ops/tests/model.rs`
- `test-corpus/`
- `ops/lib/`
- `ops/Cargo.toml` (build scripts are auto-detected; no new dependencies)

## Result

`cargo test --test corpus` lists one named test per case, e.g.
`convert__json_to_yaml__tasks_must_survive_a_json_to_yaml_conversion`.
A single case runs via `cargo test --test corpus <substring>`; unsupported
formats appear as ignored with a reason.
