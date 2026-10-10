# RFC-019: The common metadata keys are normative

## Context

The key charset makes a key guessable; it does not make two tools agree.
Both may write `due_date` and mean a date and an epoch, or `tags` and mean
a list and a comma-separated string. A key is shared only when its value
has one shape everywhere, and the common attributes — a due date, tags, a
link — are where that pays off.

Some keys cannot be shared that way. A priority scale and an estimate unit
are the tool's own: `P1`, `high`, `3`, and `5d` mean different things in
different tools, so any fixed scale is wrong in most of them.

## Decision

The common keys are normative: a document that uses one uses it in the
shape below, and a tool that reads one must understand it.

| Key | Value |
|-----|-------|
| `due_date`, `start_date` | ISO 8601 date, e.g. `2026-07-01` |
| `created_at`, `updated_at`, `completed_at` | ISO 8601 timestamp with offset |
| `tags` | list of strings, one label each |
| `assignee` | string — a name, an id, or an email |
| `location` | string |
| `url` | string — a link to the task in its source tool |
| `attachments` | list of paths — relative to the export, or a URL |

Three keys stay conventions, because their value has no shared meaning. A
tool may use them with its own scale or unit, and a reader passes the
value through instead of comparing it:

| Key | Value |
|-----|-------|
| `priority` | the tool's own scale |
| `estimates` | the tool's own unit |
| `status` | the platform's own status word, as the platform spells it |

A key enters the first table only when every tool means the same thing by
it.

## Consequences

A tool can parse a `due_date` from any document, and knows what it must
understand and what it may carry untouched. The list stays short and
honest: a key whose meaning is local is named in the second table rather
than given a scale nobody follows.

The specification carries the roster; this is why it is a rule.
