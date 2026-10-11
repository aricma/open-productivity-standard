---
version: "0"
status: open
---

# Acme product backlog

The product backlog for the Acme platform, kept in the Open Productivity Standard (OPS).

- created_at: 2026-01-01T09:00:00Z
- updated_at: 2026-07-31T18:00:00Z
- url: https://acme.example/backlog

- [ ] Fix memory leak in auth service
  Today description.
  anything: else?
  - priority: high
  - start_date: 2026-07-01
  - due_date: 2026-08-15
  - tags: ["infrastructure", "auth"]
  - status: In review
  - assignee: ada
  - [ ] Reproduce the leak
    - priority: high
    - location: Remote
  - [x] Ship the fix
    - priority: high
    - estimates: 3
    - attachments: ["notes/fix-auth.pdf"]
    - completed_at: 2026-08-15T17:00:00Z
