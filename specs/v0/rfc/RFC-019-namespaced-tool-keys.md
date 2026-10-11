# RFC-019: Namespaced keys carry tool-specific data

*Scope: v0 model.*

## Context

The common keys cover what every tool shares. A tool also tracks things
only it has — a ticket number, a board id, a sprint — and a bare key for
each would collide across tools: one tool's `id` is another tool's `id`
for something else.

## Decision

We recommend that data with no common key go under a key prefixed with the
tool's name and an underscore: `jira_ticket_id`, `github_issue_number`,
`linear_team_id`. The prefix says which tool owns the key, and a reader
that does not know the tool still sees a namespaced key and passes it
through.

## Consequences

Tool data stays out of the shared vocabulary without colliding, and a bare
key is left to the common keys.
