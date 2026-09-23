# Changelog

All notable changes to ops_lib are documented here, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The versioning
follows [SemVer](https://semver.org/). Only changes to the public
interface are listed.

> **Alpha.** Nothing has been released yet. The standard is still at
> version `0`; expect breaking changes.

## [0.1.0] - unreleased

### Added

- Reader and writer for the Open Productivity Standard (OPS).
  `read(Format, &str)` parses a document into `Vec<Task>`;
  `write(Format, &[Task])` serializes it back losslessly.
- Formats: JSON, YAML, JSONL, selected via the `Format` enum.
- The data model: `Task`, `Status`, `FlatTask`, with
  `Task::new(title, status)` and `Default` for the rest.
- The `OpsDoc` trait with one unit-struct implementation per format
  (`Json`, `Yaml`, `Jsonl`) for direct, format-typed use.
- Validation on both paths: `read` rejects invalid documents, `write`
  rejects invalid in-memory models — duplicate ids, metadata keys
  outside `^[a-z0-9_]{3,}$`, version declared on a subtask, and
  versions not among the officially released OPS versions.
- Error handling through the `Error` enum; semantic violations carry
  a `ValidationError` payload.
- `Error::code()` and `ValidationError::code()` return a stable,
  library-defined rejection code — for diagnostics and the library's
  own tests. It is not part of the OPS specification.
- Structural violations (`missing-title`, `missing-status`,
  `invalid-status`, `unexpected-field`, `invalid-field-type`) are
  reported by code instead of as JSON/YAML syntax errors.

### Security

- The library is safe-Rust only: `unsafe` is denied crate-wide and the
  panic family of lints is denied, so failures take an explicit
  `Result` path instead of escaping as an unhandled crash.
- CVE fixes will be listed here as they occur; the dependency tree is
  scanned by `cargo audit` and `cargo deny` on every CI run (see
  `SECURITY.md`).
