# Open Productivity Standard (OPS)

An open, tool-agnostic data model for exchanging tasks between
productivity tools, plus the Rust library that reads and writes it.

This is a monorepo:

- [`specs/`](specs/) — **the standard**. Start with the
  [specification, version 0 (draft)](specs/open_productivity_standard_v0.md),
  or the [spec README](specs/README.md) for the overview. Its
  [changelog](specs/CHANGELOG.md) and
  [decision records](specs/docs/adr/README.md) live there too.
- [`ops-lib/`](ops-lib/) — the **reference library** (Rust), its test
  suite, and the standard's examples and conformance fixtures under
  `ops-lib/tests/fixtures/ops/`. See the [lib README](ops-lib/README.md).

The library's tests read the examples and fixtures directly, so the spec
and its documents cannot drift apart unnoticed.

- [`TODOs.md`](TODOs.md) — open work.
- [`docs/Release.md`](docs/Release.md) — how `ops-lib` is released.
- [`maskfile.md`](maskfile.md) — local tasks (`mask setup`, `mask format`,
  `mask build`, `mask test`, `mask ci`, `mask release`).

License: MIT (see [`LICENSE`](LICENSE)).
