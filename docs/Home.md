# Home

Agent Bots is a local-first desktop control center for a personal AI workforce. One orchestrator is the interface; specialist agents do the work; Dreamer keeps durable memory.

The first complete draft now has:

- SQLite persistence for the full workforce snapshot.
- New/import project onboarding with folder checks, starter policy/README, and optional Git init.
- A real orchestrator chat and Plan & run mode.
- Generated OpenCode agents for every role.
- A task scheduler with two concurrent sessions, one owner per task, dependencies, timeouts, and retries.
- Task-to-session linking with review and accept.
- Dreamer memory proposals and approval.
- A policy gate for global and project `AGENTS.md`.
- Honest browser fallback when the desktop backend is absent.

## Run

```sh
npm install
npm run tauri dev      # desktop app
npm run dev            # browser preview only
```

Verification:

```sh
npm run check
npm test
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm run check:agents
npx tauri build --no-bundle
```

Port 1420 is strict. Stop a standalone `npm run dev` before `npm run tauri dev`.

## How to use it

Create or import a project in **Projects** first. Then open the orchestrator with `Alt+\`` or the topbar button.

- **Chat** asks the orchestrator about current work or gives a direct instruction.
- **Plan** turns a project goal into queued tasks. The scheduler starts up to two ready tasks automatically.
- Open a task to run or retry it, watch the session id, read the result, and accept it.
- Open Memory and run Dreamer to turn task evidence into pending proposals.
- Approve a proposal to write it to canonical memory.

See [[First-Draft]] for the operator walkthrough.

## Docs

- [[Architecture]] - control plane, OpenCode bridge, scheduler, security.
- [[First-Draft]] - how to operate the current build.
- [[Agent-Prompts]] - role prompts, sync, routing, policy precedence.
- [[Data-Model]] - SQLite snapshot, entities, invariants.
- [[Testing]] - checks, coverage, evaluation matrix.
- [[Design-System]] - matte dark identity, tokens, component rules.
- [[Bot-Workspace]] - legacy OpenCode workspace and mapping.

## Current limits

- SSE event updates with a 15-second polling fallback; no replay after disconnects.
- Git tasks use isolated worktrees; accept fast-forwards the base branch.
- Manual tasks are created from Tasks → New task.
- Pets: nine free-moving 56x80 desktop sprites with a shared comic speech bubble.
- Manual live model smoke passed; full Plan/task/Dreamer evaluation still pending.
- One chat session per project.

Related: [[Architecture]] · [[Data-Model]] · [[Testing]]
