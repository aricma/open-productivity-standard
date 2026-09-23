# Security

## Memory safety

The library is written in safe Rust — `unsafe` is denied crate-wide
and the panic family of lints is denied in the library, so any failure 
is reported through `Result` rather than crashing the host process.

This is a design property of the codebase, not a claim about the
parser's behavior under malformed input. OPS documents can be hostile:
please report any bug found while fuzzing or feeding malformed input.

## Dependency checks (CVE tracking)

The dependency tree is scanned for known vulnerabilities on every CI
run (locally via `mask build`, which includes the audit step):

- `cargo audit` — checks the advisory database for CVEs in the
  dependency tree
- `cargo deny` (via `deny.toml`) — license policy, plus a second
  advisory sweep

Dependabot is planned to open automated update PRs so the audit stays
green proactively rather than reactively.

## Performance and boundary tests (planned)

We intend to test the library's limits so its CPU and RAM behavior is
known and bounded:

- **Performance tests** — throughput and latency baselines for reading
  and writing large documents, per format.
- **Boundary tests** — oversized inputs (very deep nesting, very many
  records, huge metadata values, maximum line lengths), malformed
  encodings, and extreme-but-valid documents, asserting bounded
  resource use and no crashes.

Until those land, CI gates on tests, clippy, docs, and the audit —
not on runtime resource bounds.
