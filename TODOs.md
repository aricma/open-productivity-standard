# ops - todos

Open work for the OPS specification repo.

- [ ] Release ops-lib v0.1.0 (p1:e2)

## Done ✅
- [x] Split the standard into the model (`specs/v0/model.md`) and the formats (`specs/v0/formats/`), move each version's records and corpus under `specs/<version>/`, and drop Markdown from ops-lib while keeping the format in the standard
- [x] Move the Rust library under `ops/` (`ops-lib/` → `ops/lib/`)
- [x] Decide the Markdown metadata format: metadata is a bulleted `key: value`, notes stay plain lines (RFC-015)
- [x] Decide whether OPS pins a canonical byte form: it does not — comparison stays semantic, and writers must be idempotent (see `specs/v0/tests/README.md`)
- [x] Add performance and boundary tests to ops-lib: `v0::limits` guardrails enforced on read/write, `tests/security/` boundary cases, and an O(N) performance tripwire (see [ops/SECURITY.md](ops/SECURITY.md))
- [x] Implement CSV and Markdown support in ops-lib so their corpus cases are tested too
- [x] Decide how the specs handle flat task lists with no root task: a document is a forest of zero or more roots, and Markdown follows the document outline
