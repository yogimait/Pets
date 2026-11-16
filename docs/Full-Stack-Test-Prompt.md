# Full-Stack Test Prompt

Use this in a **new project** with the orchestrator in **Plan & run** mode. It is intentionally scoped enough to finish end to end, but broad enough to exercise backend, frontend, database, tests, review, and audit.

```text
Build a production-quality local full-stack app called Pulseboard in a new project folder.

Goal
A small project and task board for one local operator. It must persist data, expose a typed REST API, render a responsive React interface, and be verifiable from a clean checkout.

Stack
- Node.js 22 + TypeScript
- Express REST API
- SQLite with a checked-in schema migration (better-sqlite3)
- Zod request validation
- React 19 + TypeScript + Vite
- Lucide icons only; no large UI framework
- Vitest + Supertest for API tests
- Playwright for browser E2E

Required structure
- server/src/app.ts: Express app and middleware
- server/src/db.ts: SQLite open/migrate/seed
- server/src/routes/: separate project and task routes
- server/src/repositories/: SQL access, no SQL in routes
- server/src/validation/: Zod schemas
- server/src/errors.ts: one error envelope
- client/src/api/: typed API client
- client/src/components/: reusable UI components
- client/src/pages/: page composition
- client/src/hooks/: data and keyboard hooks
- client/src/lib/: pure formatting/filter helpers
- tests/: API and pure-logic tests
- e2e/: Playwright test
- README.md and AGENTS.md updated with exact run commands

Do not put the server, database, routes, components, and styles into one giant file.

Functional requirements
1. Seed one demo project and six tasks only on a fresh database.
2. Projects: list, create, and select.
3. Tasks: list by project, create, edit title/description/status/priority, delete with confirmation.
4. Statuses: backlog, todo, in_progress, done.
5. Priorities: low, normal, high, critical.
6. Search by title and filter by status and project.
7. Show counts: total, in progress, done.
8. Loading, empty, error, and disabled states on every data screen.
9. Responsive at 1440x900 and 1080x720.
10. Keyboard: create/edit/delete dialogs close on Escape, focus is trapped and restored, every control has a visible focus state.

API contract
- GET /api/projects
- POST /api/projects { name }
- GET /api/projects/:id/tasks
- POST /api/projects/:id/tasks { title, description, status, priority }
- PATCH /api/tasks/:id { title?, description?, status?, priority? }
- DELETE /api/tasks/:id
- All JSON bodies use one envelope: { status, statusCode, data } or { status:false, statusCode, message, error:{ code, details? } }.
- Validate at the server boundary. Return 400 for malformed input, 404 for missing records, and 409 for invalid transitions.
- No secrets, credentials, analytics, or external network calls.

Quality requirements
- Reuse components; one component per file.
- No dead controls, placeholder links, fake success paths, or unexplained dependencies.
- Add a focused test for every non-trivial branch or error path.
- Backend tests cover create/edit/delete, validation, filtering, and persistence.
- Frontend tests cover pure filtering and formatting.
- Playwright test covers: create project, create task, edit task, mark done, filter, delete.
- Run `npm install`, `npm run build`, `npm test`, and `npm run test:e2e` before declaring done.
- Record the exact commands and outcomes in the handoff.
- Do not auto-deploy, push, publish, or open external accounts.

Review and verification
The plan must use separate UI/UX, backend, frontend, testing, and audit tasks. A task is not done because files exist; it is done only when its acceptance check passes. The final audit must inspect component structure, duplication, error handling, data persistence, and the difference between claimed and actually run checks.
```

Expected result: a local app that can be started from its README, with a working API, a structured React frontend, real persistence, passing tests, and a final audit that names any remaining risk.
