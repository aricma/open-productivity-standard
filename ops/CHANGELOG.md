# Changelog

All notable changes to the `ops-lib` crate are documented here, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning follows
[SemVer](https://semver.org/). Only changes to the public interface are listed.

> **Alpha.** Nothing has been released yet. The standard is still at
> version `0`; expect breaking changes.

## [0.1.0] - unreleased

### Added

- `v0::read(Format, &str) -> Result<Vec<Task>, Error>` and
  `v0::write(Format, &[Task]) -> Result<String, Error>` parse and serialize OPS
  version-0 documents.
- `Format` supports JSON, YAML, JSONL, and CSV.
- The version-0 model: `Task` and `Status`.
- `v0::detect_version(Format, &str)` probes a document's declared version.
- `Error::code()` and `ValidationError::code()` return stable, library defined
  rejection codes for invalid documents (`missing-title`, `invalid-status`,
  `nesting-too-deep`, …).
- `v0::limits` publishes the size guardrails (documents, tasks, depth, children,
  fields, metadata); over-limit input is rejected with
  `ValidationError::LimitExceeded`.

### Removed

- Markdown. The OPS specification defines the format, but this crate does
  not claim it: `Format::Md { heading_levels }` and `Format::md()` are gone,
  and the corpus cases for Markdown are ignored.

### Fixed

- `v0::detect_version` applies the input bound, like `read`. It parsed
  whatever it was handed, so the resource guardrails could be bypassed by
  calling it instead of `read`. It stays a probe: it bounds the input and
  does not validate the document.

### Security

- Safe Rust only: `unsafe` is denied crate-wide.
