---
name: backend
description: Implement or review a service, API, database migration, auth rule, background job, or integration. Use for persistence, money, permissions, and data integrity work.
---

# Backend skill

## When to use

Use when the task changes server behavior, stored data, an external contract, or a background process. Do not use for visual-only work.

## Inputs

- acceptance criteria
- API or event contract
- schema and migration history
- callers and consumers
- deployment and rollback constraints

## Procedure

1. Read the project instructions and affected paths.
2. Map callers, consumers, jobs, and stored data.
3. Define request, response, errors, auth, and idempotency.
4. Prefer a forward-only migration. State the recovery path.
5. Validate at the trust boundary.
6. Use transactions where partial writes corrupt state.
7. Add a test for non-trivial logic and failure paths.
8. Run migrations, tests, and typecheck.
9. Report the contract, files, checks, and residual risk.

## Output

Use `prompts/shared/handoff.md`.

## Checks

- authorization is server-side
- secrets are not in code or logs
- webhooks are verified and replay-safe
- destructive migrations have a backup and rollback plan
- error envelope matches the project

## Boundaries

Do not deploy, run production mutations, use production data, or weaken validation to pass.
