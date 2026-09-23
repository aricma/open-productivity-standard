# ADR-004: `version` is optional and lives on the root

**Status:** Accepted.

## Context

A version names the spec a document is modeled after. Without one, a
document follows the latest released version, so writing task data and
building tools stays easy while the specs develop; a document can still
pin the exact version.

## Decision

`version` is optional, must be a released OPS Specification version, and
can only be set on root tasks.

## Consequences

If no version is set and the latest version has deprecated features the
dataset uses, tools must migrate or fail. A document can hold multiple
roots with different versions.
