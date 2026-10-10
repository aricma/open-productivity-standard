# RFC-025: A metadata value may contain line breaks

## Context

JSON and YAML hold a line break inside a value naturally; Markdown and CSV
do not, because their structure is one line. Whether a value can be
carried therefore depends on the format, and a writer that emits a raw
line break into a Markdown bullet or an unquoted CSV field breaks the
structure and reads differently in every tool.

## Decision

A metadata value may contain line breaks. A writer encodes them in the
format and never writes a raw line break the format cannot hold: CSV
quotes the field per RFC 4180, and Markdown writes the value on one line
as a double-quoted string with the break escaped (`\n`, JSON-style). A
reader decodes the value back to the same text, line breaks included.

## Consequences

Every value the model can hold is writable in every format, so no format
is lossier than another for this, and a value reads back exactly as it was
written.
