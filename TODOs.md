# ops - todos

Open work for the OPS specification repo.

- [ ] Release OPS v1, promoting the v0 draft (p1:e2)
- [ ] Implement CSV and Markdown support in ops-lib so their corpus cases are tested too (p2:e3)
- [ ] Add the planned performance and boundary tests to ops-lib — throughput/latency per format, and oversized/malformed inputs with bounded resource use (see [ops/SECURITY.md](ops/SECURITY.md)) (p2:e3)
- [ ] Decide how the specs handle flat task lists with no root task (p2:e2)
- [ ] Decide whether OPS pins a canonical byte form per serialization so
  writer output can be compared byte for byte (see `test-corpus/README.md`) (p1:e2)
- [ ] Add a CLI at `ops/cli` with a simple interface to validate and auto-fix task documents in any supported format, for CI and for AI agents (p2:e3)
- [ ] Add a `SKILL.md` so agents can use the ops CLI to format, fix, and validate local `TODOs.md` files and convert them to any supported format (p2:e3)

## Done ✅
- [x] Move the Rust library under `ops/` (`ops-lib/` → `ops/lib/`)
- [x] Decide the Markdown metadata format: metadata is a bulleted `key: value`, notes stay plain lines (ADR-011)
