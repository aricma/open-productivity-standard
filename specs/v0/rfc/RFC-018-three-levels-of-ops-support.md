# RFC-018: Three levels of OPS support

*Scope: v0 model.*

## Context

The model requires only `title` and `status` (DR-001); every other field —
`id`, `notes`, `subtasks`, `metadata` — is optional. That is what makes
OPS cheap to adopt: a tool that has none of them can still read and write
tasks, and DR-001's consequence — "any tool can always write and read a
valid document without inventing data" — depends on it.

But the Conformance section describes only one thing: full support. It
lists reader, writer, and transformer obligations, and the transformer
must "preserve all valid `metadata` verbatim". Read literally, supporting
OPS is all-or-nothing: either a tool carries the whole data model, or it
does not support OPS. DR-001's promise that any tool can exchange tasks is
then untrue for every tool that does not.

That is the wrong bar for the users OPS exists for. A tool that exports
its tasks grants data freedom even if it exports nothing else. A tool that
also imports them lets a user migrate. Only a tool promising a lossless
round trip has to carry the whole model.

## Decision

OPS defines three levels of support. A tool claims the highest level it
meets, for each serialization it claims to support.

1. **Export core.** The tool writes valid OPS documents of the core model:
   `title` and `status`, plus `id`, `notes`, and `subtasks` where it has
   them. It may omit `metadata`.
2. **Import and export core.** The tool also reads core documents and
   derives the tasks from them, preserving the core model — including the
   hierarchy in `subtasks`. It may discard `metadata`.
3. **Full — zero data loss.** The tool supports the core model plus
   `metadata`: it reads and writes every field and preserves every node,
   every relation, and all valid `metadata` verbatim across an
   import → export cycle.

The levels build on the reader, writer, and transformer obligations, which
still apply at every level: a reader accepts every valid document and
rejects every invalid one, a writer emits only valid documents, and a
transformer does both. A Level 1 tool is an OPS Writer; Level 2 adds the
OPS Reader; Level 3 adds the OPS Transformer.

What a Level 1 or Level 2 tool handles is the core model, not just the
required fields. `metadata` is optional to preserve below Level 3
and required at it. Conformance still never means understanding metadata:
no tool is required to know what another tool's metadata means, or to
support every serialization.

## Consequences

A tool adopts OPS at the level it can afford — export for data freedom,
import plus export for migration, a lossless round trip for a peer — and
none of those is "not OPS". The full model stays a requirement only of
tools that promise zero data loss.

The cost is more to state: support is three levels per format, not one
badge, and a list of tools has to say which level each supports. It also
makes an asymmetry explicit that was always true — a Level 2 tool can read
a Level 3 tool's document and lose data on the way back out.
