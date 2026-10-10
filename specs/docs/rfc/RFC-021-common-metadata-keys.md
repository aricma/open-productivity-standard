# RFC-021: The common metadata keys are recommendations

## Context

The key charset makes a key guessable; it does not make two tools agree.
Both may write `due_date` and mean a date and an epoch, or `tags` and mean
a list and a comma-separated string. A reader only carries metadata and
does not interpret it, so no tool can be forced to use a key and no
reader can demand one. What we can do is recommend spellings for the
attributes most tools share, so two tools that both use one line up.

## Decision

We recommend the keys and shapes below for the common attributes. A tool
uses one only when it has that data, and a reader passes the value
through.

| Key | Value |
|-----|-------|
| `due_date`, `start_date` | ISO 8601 date, e.g. `2026-07-01` |
| `created_at`, `updated_at`, `completed_at` | ISO 8601 timestamp with offset |
| `tags` | list of strings, one label each |
| `assignee` | string — a name, an id, or an email |
| `location` | string |
| `url` | string — a link to the task in its source tool |
| `attachments` | list of paths — relative to the export, or a URL |

Three keys are also common but have no shared meaning, so the value is
the tool's own:

| Key | Value |
|-----|-------|
| `priority` | the recommended `A`–`D` scale (RFC-020), or the tool's own value under a namespaced key (RFC-019) |
| `estimates` | the tool's own unit, or a namespaced key |
| `status` | the platform's own status word, as the platform spells it |

Data that fits no common key goes under a namespaced key (RFC-019). A key
enters the first table only when every tool means the same thing by it.

## Consequences

A tool that wants to interoperate has a spelling to use, and one that has
its own keeps its own — under a namespaced key, where it cannot collide.
Nothing here is enforced; these are recommendations, not rules.
