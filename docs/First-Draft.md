# First-Draft Operator Guide

This is the current end-to-end flow. It is a first draft: the local control plane and task/session bridge are real; a manual live runtime smoke passed, but the full Plan/task/Dreamer evaluation has not been run yet.

## 1. Create or import the project

Open **Projects** and click **New project**. Project setup is app-level, not orchestrator-level.

| Starting point | What to do |
|---|---|
| Folder exists, nothing started | Enter name, key, and absolute path. Leave folder creation off. Enable the starter `AGENTS.md` if the folder has none. |
| Mid-project folder | Enter the same information. The existing `AGENTS.md`, README, and files stay in place; only missing starter files are added. |
| Idea only, no folder | Enter `D:\Projects\<name>`, enable **Create the folder**, optionally create starter `AGENTS.md`/`README.md`, and optionally initialize Git. |

The form checks the path before saving when you press **Check**. It never overwrites an existing `AGENTS.md` or `README.md`. A project without `AGENTS.md` is created but cannot run real work.

OpenCode skills and MCP servers are inherited from your OpenCode configuration. MCP tool calls are set to **allow**, so OpenCode runs them without a prompt. The Task detail permission panel remains a fallback for unexpected non-MCP requests. Check **Settings > OpenCode runtime** for MCP connection status before a task that depends on one.

## 2. Start the desktop app

```sh
npm run tauri dev
```

Use the desktop build, not the browser preview, for real data. The browser keeps the fixture snapshot and scripted chat. Model credentials stay in OpenCode; the app never reads or stores provider keys.

On first launch the app seeds `workforce.db` with the fixture workspace. Seed tasks get `autoStart: false`, fake running/review tasks become queued, agents start idle, and the scripted chat, seed activity, and sample proposals are cleared. No task and no model session starts until you explicitly run or plan work. Later launches read the stored snapshot. A pre-runtime database is migrated once: seed auto-start is disabled, fake sessions are cleared, and plan-created tasks are preserved. The database lives in the Tauri app data directory.

## 3. Open the orchestrator

Press `Alt+\`` or click **Orchestrator**.

- Select the project you created. `CORE` is this repository. The seeded `VLT`, `FLD`, and `LCH` projects are display-only and their paths may not exist.
- **Chat** sends a real prompt to `orchestrator-workforce`.
- **Plan & run** asks the orchestrator for a task plan. The reply is parsed as JSON and saved as queued tasks; the scheduler then starts up to two ready tasks.

The browser preview shows `simulated`. The desktop app shows `local`.

## 4. Understand Plan & run

Plan mode is intentionally narrow:

1. The orchestrator receives the project context, existing tasks, and available specialist ids.
2. It must return one JSON object with a summary and tasks.
3. The app validates owner ids and creates tasks with `autoStart: true`.
4. The scheduler starts up to two ready tasks on the next tick.
5. Each task gets its own OpenCode session and result.

Plan mode does not create a project or change policy. It adds tasks to the selected project.

## 5. Watch tasks

Open **Tasks**. A planned task moves through:

```text
queued -> running -> review -> done
```

- Open a task to see its acceptance criteria, dependencies, session id, and result.
- **Run now** starts a queued task manually.
- **Retry** restarts a blocked task.
- **Abort** interrupts a running session and blocks the task.
- **Accept result** marks a review task done. Dependent tasks become eligible on the next tick.

The scheduler enforces:

- two running tasks maximum
- one running task per owner
- all dependencies done before start
- a policy gate before any model call
- a 30-minute blocked timeout

## 6. Use Dreamer

Open **Memory**.

1. Select the project.
2. Click **Run Dreamer**.
3. Dreamer reads review/done task results and existing memory.
4. It returns findings and pending proposals.
5. Open a proposal and approve or reject it.

Approving applies the proposal to canonical memory. The next task prompt includes up to eight approved entries for that project.

## 7. What to expect from the orchestrator

There is one orchestrator role, not one orchestrator per folder. A project is a scope, not a separate brain. The selected project controls the working folder, policy, context, tasks, and memory. The orchestrator can read the selected project and the app snapshot, but it does not have blanket access to the rest of the PC.

The orchestrator is a manager, not a free-running swarm:

- It plans and reports.
- It does not chat directly with specialists.
- It does not bypass the task board.
- It does not run without a user action.
- It stops when tasks are done or blocked.

The app mediates all work. That is what keeps the workflow auditable and prevents two agents from claiming the same task.

## 8. What is intentionally not done

- Full live Plan/task/Dreamer evaluation with provider keys.
- SSE streaming.
- Git worktrees for coding tasks.
- Manual task creation form. Project creation/import is implemented.
- Desktop pet habitat.
- A permission approval queue for unattended task sessions.
- Normalized SQLite tables.

Do not use this draft on an untrusted repository with broad shell permissions yet.

## 9. Next evaluation

Run the matrix in [[Testing]] once the provider keys are approved:

1. Smoke: fresh database, seed once, seven views.
2. Chat: prompt and reply survive restart.
3. Plan: one valid task, no duplicates.
4. Execution: queued -> running -> review.
5. Accept: review -> done, dependent starts.
6. Concurrency: three tasks, max two running.
7. Failure: missing path/policy becomes blocked with a clear message.
8. Memory: Dreamer proposal approved once and injected into the next task.
9. Security: `.env`, external paths, subagents, destructive shell denied.
10. Restart: state survives; stale session recovers.

Related: [[Home]] · [[Architecture]] · [[Testing]] · [[Data-Model]]
