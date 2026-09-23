# Security

**TL;DR** — safe Rust (`unsafe` denied) with exhaustive error handling:
Rust has no exceptions, so every failure is an explicit monad and the
panic lints keep it from turning into a crash. And the standard's
conformance cases are plain text, portable to any OPS implementation
for external validation.

## Memory safety

The library is written in safe Rust. `unsafe` is denied crate-wide
(`[lints.rust]`), so it contains no unchecked pointer
arithmetic, no manual allocation, and no memory unsoundness of its own.

## All branches are handled

Rust has no exceptions, so failure is a value rather than a crash:
every fallible operation returns a monad that the caller must handle,
and panics are denied in library code. Invalid or hostile documents —
malformed syntax and semantic violations alike (duplicate ids, bad
metadata keys, misplaced versions) — take an explicit error path. This
is a property of the code, not a proof that the parser handles every
adversarial input; please report anything found while fuzzing.

## Portable conformance corpus for external validation

The standard ships a corpus of plain-text, format-neutral documents
that any implementation of OPS — in any language — can run:

- well-formed examples that must round-trip unchanged,
- valid documents that must parse and import cleanly,
- invalid documents that must be rejected by a conforming parser.

Our Rust suite reads the corpus in place; nothing about it is
Rust-specific. Running the same corpus is how an external validator
compares its behavior against the reference implementation, on the
cases the reference is held to. The CSV and Markdown cases are specced
but not yet implemented in the library, so the Rust tests do not cover
them yet — they remain portable cases any other implementation can
already run.

## Dependency checks (CVE tracking)

The dependency tree is scanned for known vulnerabilities on every CI
run (locally via `mask build`, which includes the audit step):

- `cargo audit` — checks the advisory database for CVEs in the
  dependency tree
- `cargo deny` — license policy, plus a second advisory sweep

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
