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

## Resource limits and boundary tests

The library enforces explicit, documented guardrails so a document
cannot make it allocate without bound or recurse until it crashes.
These are **ops-lib policy, not part of the OPS standard**: another
implementation may choose different numbers, and a document over one of
these is rejected here, not declared non-conformant.

| Guardrail | Value |
|---|---|
| whole document | 16 MiB |
| tasks per document | 100 000 |
| nesting depth (root = 0) | 256 |
| subtasks per task | 10 000 |
| title | 1 024 bytes |
| notes | 65 536 bytes |
| id | 256 bytes |
| version | 16 bytes |
| metadata entries per task | 64 |
| metadata key | 64 bytes |
| metadata value, serialized | 65 536 bytes |

Each is a constant in `lib/v0/limits.rs` with a comment explaining the
number. Over-limit input is rejected with `ValidationError::LimitExceeded`
and a stable `code()` (`nesting-too-deep`, `field-too-long`, …), never a
panic or an abort. `read` checks the document size before parsing and the
structural limits while building; `write` runs the same validation before
serializing, so a deep in-memory tree is rejected instead of overflowing
the stack.

The limits are covered by `tests/security/` (generated boundary cases,
including the deep-chain and deep-write regressions) and
`tests/performance/` (read/write scaling with an O(N) tripwire), run by
`mask test --profile security` and `mask test --profile performance`.
