# tests

The integration test suite. Each `tests/*.rs` file is its own crate,
compiled with the library's public API only.

## Test files

| File | What it covers |
|---|---|
| `ops_conformance.rs` | The OPS conformance contract, at the level of rough guidelines: every supported example round-trips without changing, every `valid/` fixture round-trips without failing, every `invalid/` fixture is rejected. It runs the standard's own documents in place, so drift fails CI. No error-class checks here. |
| `read.rs` | Our own `local/` fixtures: a few valid documents pinning the full Task model per format, and many small invalid documents pinning every error across formats and nested/flat forms. |
| `roundtrip.rs` | The lossless contract for the full model: write → read → identical model in every supported format (json/yaml/jsonl). |
| `write.rs` | The writer: byte-stable output (writing a model reproduces its fixture character for character), flat forest + empty roundtrips, and validation of in-memory models before serializing (semantic rules + single-root rule). |

Tests are table-driven: each case is a `TestCase { message, given, expected }`
row run through a shared runner in `common/`.

## `tests/common/` — shared scaffolding

Each test binary pulls in the whole `common/` module tree but uses only a
subset, so `dead_code` is allowed crate-wide (Cargo.toml `[lints.rust]`).

| Module | Contents |
|---|---|
| `test_case.rs` | The `TestCase<Given, Expected>` table shape. |
| `expectation.rs` | `Expectation` — what a case must produce (`Model(Vec<Task>)` or `Error(ErrorKind)`). |
| `error_kind.rs` | `ErrorKind` — the error classes worth asserting (payloads ignored except for `Validation`). |
| `round_trip_test_case.rs` | `RoundTripTestCase` — one document that must survive or fail the read → write → read pipeline. |
| `run_test_cases.rs` | The runners: `run_test_cases` (generic), `run_read_test_cases`, `run_write_roundtrip_cases`, `run_roundtrip_passes` (fixed-point, byte-exact), `run_roundtrip_fails`. |
| `fixtures.rs` | Fixture paths (`OPS_TESTS`, `OPS_EXAMPLES`, `LOCAL`), loading (`read_fixture`), file listing (`files_under`, `get_all_ops_*`), and case building (`fixture_paths_to_roundtrips_test_cases`). |
| `models.rs` | Model builders matching the standard's examples (`example_model`, `work_model`, …) plus `full_model` — the maximal Task, pinned by the local valid fixtures. |

## Fixtures — `tests/fixtures/`

Two trees:

- `ops/` — the standard's own documents, moved out of `specs/` so the
  library tests them directly (there is no vendored copy and no
  freshness job any more). `examples/` holds every example;
  `tests/{json,jsonl,yaml,csv,markdown}/{valid,invalid}/` holds the
  conformance fixtures. Only the formats the library supports
  (json/yaml/jsonl) are walked today; the CSV and Markdown sets wait for
  support (see the root `TODOs.md`).
- `local/` — our own fixtures, for cases the standard does not cover:
  - `valid/` — documents that must parse, one per supported format,
    carrying the entire Task model (`full-model.{json,yaml,jsonl}`, the
    byte-exact write reference).
  - `invalid/` — one small focused document per (error × format ×
    nested/flat) combination, each breaking exactly one rule.

Keep invalid fixtures small and single-purpose: the case message states
the expected error, the fixture only needs to trip it.
