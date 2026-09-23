# ADR-010: CSV spreads metadata over columns

**Status:** Accepted.

## Context

CSV is the flat, human-readable streaming format. Dumping the whole
`metadata` object into one JSON-encoded cell makes rows unreadable to CSV
clients and hides simple dates, priorities, and locations behind escaping.
Most metadata is flat string key-value pairs, so it maps naturally onto
columns; only rare complex values need JSON.

## Decision

CSV reserves the top-level field names as columns and treats every other
column as metadata. A metadata column is the key, optionally prefixed
`metadata_` to avoid a collision, and optionally suffixed `_json`,
`_string`, `_number`, or `_bool` to pin a type. A cell that parses as JSON
is that value, and any other cell is a string. The optional `metadata`
JSON column is allowed and merged with the metadata columns, which
override it. Invalid metadata keys still make the document invalid
(rule 5).

## Consequences

Simple metadata is directly readable and writable by CSV clients; complex
values still round-trip; unknown columns add data instead of failing the
read.
