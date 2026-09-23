# ADR-002: Only `title` and `status` are required

**Status:** Accepted.

## Context

The model's core is a line on paper with a checkmark. Requiring more — an
id, timestamps, metadata, a version — would force tools without such data
to invent placeholder values on export, and every other tool to guess
whether the value is real.

## Decision

Every task carries exactly `title` (string) and `status` (enum); all other
fields are optional.

## Consequences

Any tool can always write and read a valid document without inventing
data.
