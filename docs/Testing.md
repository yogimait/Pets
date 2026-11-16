# Testing

## Commands

| Command | Purpose | Current result |
|---|---|---|
| `npm run check` | Svelte and TypeScript diagnostics | 0 errors, 0 warnings |
| `npm test` | Agent sync validation plus data, selector, prompt, runtime-format, and Markdown tests | 28 Node tests pass |
| `npm run check:agents` | Validate generated OpenCode agent files | 9 agents pass |
| `npm run build` | Production renderer bundle | Passes, about 281 KB JS / 33 KB CSS |
| `cargo check --manifest-path src-tauri/Cargo.toml` | Rust type check | Passes |
| `cargo test --release --manifest-path src-tauri/Cargo.toml --lib` | Rust store, JSON extraction, dependency, scheduler, seed/migration, project validation, starter files, permission parsing, SSE frames | 26 tests pass |
| `npx tauri build --no-bundle` | Release application compile | Passes; `agent-bots.exe` built |
| `npx tauri build --bundles nsis` | Windows installer | Passes; `Agent Bots_0.1.0_x64-setup.exe` (~3.46 MiB) |
| `impeccable detect src --json` | Design detector | 0 findings |

Use `$env:CARGO_BUILD_JOBS=2` on low-RAM Windows machines.

## Node test coverage

`tests/data.test.ts` and `tests/prompts.test.ts` cover:

1. Task owners and projects resolve.
2. Activity and memory references resolve; confidence is bounded.
3. IDs are unique per collection.
4. Current tasks are running and owned by the right agent.
5. Agent models resolve to configured providers.
6. Metrics match task statuses.
7. Project and step progress stay in range.
8. Filters return strict subsets.
9. Chat reference chips point at real records.
10. Proposals and Dreamer runs reference real records.
11. Every agent points at a registered prompt.
12. Every expected role has exactly one prompt.
13. Prompt assembly preserves policy and role order.
14. Specialists receive the handoff contract; Dreamer receives the memory contract.
15. Orchestrator routing covers every specialist.
16. Agent OpenCode ids are unique.
17. Task dependencies resolve and never self-reference.

`tests/runtime.test.ts` covers runtime message normalization, malformed entries, permission normalization, and model key encoding.

`tests/markdown.test.ts` covers table parsing, fenced code, headings, lists, inline code, and bold segments. Rendered output uses text nodes only; raw HTML is never injected.

`scripts/check-agents.mjs` checks that every generated OpenCode agent exists in both the project and global agent directories, with frontmatter, primary mode, a provider/model pair, permissions, a substantial body, no legacy taskboard command, no secret-like literal, MCP allow rules, and a destructive `Remove-Item` deny rule.

## Rust test coverage

`src-tauri/src/workforce.rs` tests:

- Extract the first JSON object from prose and markdown fences.
- Ignore braces inside JSON strings.
- Reject incomplete JSON.
- Require every dependency to be done.
- Parse `provider/model` pairs.
- Seed the store once and persist updates.
- Exclude orchestrator and Dreamer from planner owner ids.
- Map an app agent to its OpenCode agent.
- Select only ready `autoStart` tasks.
- Prevent late review/blocked transitions from overwriting terminal tasks.
- Normalize seeded fake sessions into queued tasks, disable seed `autoStart`, and idle agents.
- Migrate pre-runtime seed data without touching Plan task ids.
- Keep real chat while clearing stale seed sessions during the v2 to v3 migration.
- Validate project keys and normalize folder identity.
- Create starter policy/README files only when missing.

## Live model smoke check

The release app connected to the shared OpenCode service and started a real session with:

```text
agent: orchestrator-workforce
model: opencode-go/deepseek-v4.1-flash
prompt: Reply with exactly the single word READY. Do not use any tools.
```

The assistant returned `READY`. The session showed the custom agent list and the six configured MCP server names. No tool or MCP action was used. This confirms the app → shared OpenCode service → model path.

App-level smoke on the `Live Test` project:

- chat returned `READY. This is the Live Test project...`
- Plan & run created one task owned by `scout`
- the task used a real session and returned a review result
- the first attempt failed with `Agent not found: orchestrator-workforce`; the cause was project-local agent files. Agents are now installed globally and the same `Live Test` project works.

MCP auto-allow is recorded below.

## Desktop smoke check

The release binary was launched once without starting a model session. SQLite was created at `%APPDATA%\com.agentbots.app\workforce.db` and migrated to schema 3. The stored snapshot had 30 tasks, 0 running, 0 `autoStart`, 0 session links, 0 active agents, and empty chat/activity/proposals/runs. This confirms the first-launch/migration path does not trigger work.

After switching the adapter to the shared OpenCode background service, the release runtime panel reported the custom agents (`orchestrator-workforce`, `pixel`, `forge`, `scout`, `voice`, `probe`, `iris`, `verity`, `dreamer`) and the six configured MCP servers (`jev`, `laya`, `n8n-mcp`, `playwright`, `rea`, `stitch`). Names are listed even when a server is failed or pending; the dot and hover text carry the status. No private OpenCode child was left behind.

The New Project sheet was then driven through WebView2 CDP:

- entered a temporary `D:\Projects\agent-bots-onboarding-test` path
- created the folder
- created starter `AGENTS.md` and `README.md`
- saved the project and opened its detail sheet
- verified the project in SQLite
- restored the pre-test database and removed the temporary folder

No model session was started.

### Chat permission fallback

Before the switch to MCP `allow` rules, `Live Test` chat asked Playwright to open `https://example.com` and report the title.

- the chat session did not return the old pending-permission error
- `browser_navigate` and `browser_evaluate` were approved once automatically
- the reply reported `Example Domain`
- no browser action beyond navigation, title read, and close ran

The fallback remains for any non-MCP permission request.

### Restart persistence

Killed and reopened the app with the existing SQLite file.

- before restart: 28 chat messages, 35 tasks, 17 memory entries, 2 pending proposals
- after restart: same counts; task statuses unchanged; no task auto-started
- `LT` approved memory and the two pending proposals appeared in the UI
- chat after restart returned exactly `RESTART OK`
- the old in-memory chat session was gone; OpenCode created a new one without a stale-session error
- chat history grew to 30 messages

Stale recovery for a **running task** with a dead session id is not tested.

### Dreamer live run

`Live Test` Dreamer run `run-1794723213429-1` completed with 3 proposals:

- approved `Playwright MCP writes snapshots into .playwright-mcp/`
- memory entry `mem-1794723252912-1` was written with project scope `lt`
- the next Scout task prompt contained the entry under `# Approved project memory`
- that task returned `# Live Test project rules` and was accepted

Two proposals remain pending. Reject, update, delete, and merge proposal actions are not tested.

### MCP auto-allow

Generated agents now use `allow` for `playwright_*`, `n8n_mcp_*`, `stitch_*`, `rea_*`, `jev_*`, and `laya_*`.

`Live Test` task `t-1794721893825-0` (Probe) opened `https://example.com`, reported the title and final URL, and closed the browser.

- no **Permission needed** panel appeared
- no manual approval was required
- the task reached review
- **Accept result** moved it to done

The Task detail permission panel remains as a fallback for unexpected non-MCP permission requests. An earlier ask-mode round-trip was verified before the switch.

### Task failure and retry

- A task assigned to the missing `vault` project path blocked with `Project directory does not exist: D:\Projects\vault-api`; no session was created and no loop started.
- A task on `Live Test` blocked with `Project policy is missing: ...AGENTS.md` after the policy was temporarily moved aside.
- After restoring `AGENTS.md`, **Retry** created a real session and returned `RETRY OK`; the task reached review.

### Stale running session

The first run exposed a real gap: `workforce_poll` checked permissions before connecting to OpenCode, so a stale task blocked with `Permission check failed: OpenCode service is not connected`.

Fix:

- resolve the task project and call `runtime.ensure()` before checking the session
- fetch the session before the permission list
- block with `Session check failed: ...` when the session is gone

Retest with a fake session id blocked with:

```text
Session check failed: OpenCode 404 Not Found: SessionNotFoundError
```

### Concurrency

Injected three ready tasks: two owned by Pixel, one by Forge.

```text
Pixel A: running
Pixel B: queued (same owner)
Forge C: running
```

Two running tasks maximum, one per owner. Test tasks were removed afterward.

### SSE event bridge

- Rust subscribes to OpenCode `/api/event` with loopback basic auth.
- The runtime log records `SSE connected` and each forwarded event as `SSE event <type> session=<id>`.
- Session/message/permission events are filtered and emitted as `workforce-event`.
- Frontend sync debounces the event by 150ms, then runs tick + poll.
- 15-second polling remains as fallback because the OpenCode stream is volatile.
- A Plan-created task started 5.8s after creation, below the 15-second fallback, consistent with event-driven start.

To inspect it:

```powershell
Get-Content "$env:TEMP\agent-bots-runtime.log" -Tail 30
```

Expected at minimum:

```text
SSE connected
SSE event server.connected session=-
SSE event session.created session=ses_...
```

OpenCode may not emit every declared event type. If `session.idle` is missing, the 15-second fallback closes the task.

SSE reconnect under long active turns and event loss across disconnects are not load-tested.

### Real empty workspace and Dreamer context

- automatic memory review approves proposals at 80%+ and rejects below 80%; unit-tested at the threshold
- agent display names are the pet names and agent cards use the pixel sprites
- current database uses three projects (`core`, `lt`, `sexyc`), twelve done tasks, six approved memory entries, and decided proposals after the automatic threshold pass
- demo projects Vault, Field Notes, and Launch Page removed
- browser preview is read-only and no longer scripted
- fixtures remain only for unit tests
- Dreamer run on `core` read the project markdown and DevVault hub and produced 7 pending proposals
- one real proposal was approved; memory count is 1, last write shown correctly
- the existing Memory page no longer renders `Invalid Date` with an empty ledger

### DevVault maintenance access

Verified that a task prompt contains:

- the DevVault root
- `DevVault/Home.md`
- the project hub path
- the rule to update them after meaningful work and not on read-only/no-change work

A read-only Scout task read `C:\Users\Hp\Obsidian\DevVault\Projects\Agent-Bots.md` and returned `# Agent-Bots`.

A second Scout task on `Live Test` made a small README change, created `DevVault/Projects/Live-Test.md`, and added a Live Test row to `DevVault/Home.md`. It reported the exact files changed, and no other DevVault notes were touched. The task was accepted.

Generated agents allow `external_directory` for DevVault only. A meaningful update on a real code project has not yet been run.

### Security denials and hardening

Chat probe:

- `.env` read denied
- `C:\Windows\win.ini` read denied as `external_directory`
- subagent and shell tools unavailable

Task probe found a real hole: `Remove-Item -Recurse -Force <empty temp dir>` was **allowed** because the old deny pattern `Remove-Item * -Recurse*` did not match that command order. The empty test directory was deleted.

Fix: generated agents now deny all of `*Remove-Item*`, `Remove-Item *`, `del *`, `rmdir *`, `rm -rf *`, destructive Git commands, `format`, shutdown, and restart patterns.

Retest:

```text
Remove-Item -Recurse -Force D:\Projects\agent-bots-hardening-temp
```

was denied with `Permission denied: shell`; the directory and `keep.txt` remained. `scripts/check-agents.mjs` now fails if the Remove-Item deny rule disappears.

Shell hardening also denies `Invoke-Expression`, `iex`, and PowerShell encoded-command forms. Residual risk: shell deny rules are pattern-based, not a sandbox. A command that does not match a listed pattern can still run.

### Manual task form

Created a task from **Tasks → New task** with project `CORE`, owner Pixel, title, description, acceptance, and start-now unchecked.

- task appeared as `manual`
- status was `queued`
- `autoStart` was `false`
- backend validation rejected empty title/description paths by construction
- test task removed afterward

### MCP health preflight

Manual tasks can require MCP servers; task text also infers `playwright`, `n8n`, `stitch`, `rea`, `jev`, or `laya` by exact token.

Before a worktree or session is created, the backend checks the required servers at the task location and waits up to 20 seconds for pending servers. Failed or still-pending servers block the task with a clear error.

Unit coverage: required-MCP inference. The all-connected path was exercised live; a failed-server path was not forced because all six servers were connected.

### Provider failure UX

Provider errors containing `401`, `unauthorized`, `authentication`, `credential`, `api key`, or `not configured` now return:

```text
Provider credentials for … are missing or invalid. Configure the provider in OpenCode, then retry.
```

Other model-start failures remain the generic model error. Unit coverage exists; a live credential failure was not forced.

### Desktop pets

Opened the pets from the main topbar.

- nine transparent 56x80 pet windows with named pixel sprites
- no visible frame or box
- each agent has a distinct idle area and working area
- idle pets wander slowly with pauses; active pets move faster around the work area
- drag/hover surface is exactly the 56x80 sprite; the transparent window margin does not start a drag or show the bubble
- drag Micheal from the lower desktop to an upper position: the window followed the pointer, stayed at the drop point, then wandered around that new anchor
- started a real Live Test task owned by Scout: Ryan moved from his idle area to the work area while the task was active, returned after review, and the task was accepted as done
- hover/click opens one shared comic speech bubble with name, role, status, current task, and latest activity
- the main app does not open from a pet
- bubble hides when the pointer leaves the pet, after 4 seconds, or via close
- topbar Alerts shows pending review results; Review next opens the task sheet with Accept result, Reject result, and Edit task
- edit opens title, description, acceptance, priority, and owner fields; cancel leaves the task unchanged
- accepting the final task appends a project-complete report and sends the desktop notification
- topbar `Pets` hides all nine windows; `Show pets` restores all nine and unhides motion
- hidden pets pause movement; showing them resumes from the last position
- closing the main window exits the app process and removes all pet windows
- launching the app while it is already running exits the second process, focuses the existing main window, and does not add pet windows
- topbar Pets re-shows all pet windows

### Packaged agents and installer

- the release binary embeds the nine generated agents
- startup installs them into `~/.config/opencode/agent` when the file is missing or still Agent-Bots-generated
- verified by removing global `pixel.md` and relaunching; the app recreated it
- NSIS installer built at `src-tauri/target/release/bundle/nsis/Agent Bots_0.1.0_x64-setup.exe`

Installer launch, upgrade, and uninstall are not tested.

### Git worktree isolation

Created a temporary clean Git repo with `AGENTS.md` and `README.md`, added a temporary app project and read-only Scout task.

- task ran in `appdata/worktrees/<task-id>`
- branch was `agent-bots/<task-id>`
- session id and worktree path were stored on the task
- Accept fast-forwarded the base branch and removed the worktree registration/branch
- task moved to done

Not tested live: dirty-base rejection and merge conflicts. Accept now uses `git merge --no-ff`; a conflict returns a clear error with the retained worktree path and branch, and the task stays in review for a retry. The worktree directory can remain empty briefly while OpenCode releases its cwd handle.

### Visual and keyboard

- 920x680 default overlay verified
- 640x480 floor verified; composer, controls, and table output fit; no page overflow
- a long draft caps the composer at 160px and scrolls inside the field instead of covering the thread
- long Markdown table wraps instead of forcing horizontal page overflow
- Esc hides the overlay; `Alt+\`` shows it again
- main window at 1440x900 and 1080x720 layouts checked
- Impeccable reports 0 findings for the overlay and Markdown component

### Orchestrator output formatting

Assistant messages now render a small safe Markdown subset: headings, paragraphs, lists, fenced code, inline code, bold text, blockquotes, rules, and tables. Raw HTML is rendered as text, never injected. Added `tests/markdown.test.ts`.

## MCP prerequisite

The Playwright MCP must start inside the 30 second MCP startup budget. A cold `npx -y @playwright/mcp@latest` start can exceed that budget and the server is then missing from the task session even though the runtime panel lists its name. Install it once and point the global MCP command at the installed shim:

```sh
npm install -g @playwright/mcp
```

```json
"playwright": {
  "type": "local",
  "command": ["playwright-mcp"],
  "enabled": true
}
```

The runtime panel shows a yellow dot for failed or pending MCP servers; hover the name for the status and error. Playwright must show a green dot before an MCP task test.

## What is not tested yet

- Live Tauri IPC against the real SQLite file in the packaged app beyond the app's own commands.
- MCP auto-allow with mutating browser actions such as click or form fill.
- Unexpected non-MCP permission requests during unattended task runs.
- Long-running concurrent tasks under model load.
- SSE reconnect during a long active turn and any event loss across disconnects.
- Worktree merge conflicts and dirty-base rejection are not live-tested.
- Provider credential failure.
- Installer launch, upgrade, and uninstall.
- DevVault maintenance by a task on a real production codebase; the write test used `Live Test`.
- Shell commands outside the current destructive-pattern list; pattern matching is not a sandbox.
- Packaged installer flow; verification uses the `--no-bundle` release binary.

## Evaluation matrix for the next pass

| Area | Test | Pass condition |
|---|---|---|
| Onboarding | Create a project in an existing folder, a mid-project folder, and a brand-new folder | Folder path validates; starter files are only created when missing; project appears in selector |
| Smoke | Launch desktop app with a fresh data directory | Empty real workspace seed writes SQLite once; all seven views render |
| Orchestrator chat | Send a short prompt to `core` | Reply is stored in chat and survives restart |
| Plan | Plan one small task | JSON parses; one queued task appears; no duplicate plan tasks |
| Execution | Run a read-only task | Task moves queued -> running -> review; session id stored |
| Review | Accept the result | Task is done; dependent task becomes eligible |
| Concurrency | Queue three tasks | At most two run; no owner has two running tasks |
| Failure | Use a missing project path or missing policy | Task or action is blocked with a clear error, no loop |
| Memory | Run Dreamer, approve one proposal | Proposal applies once; memory appears in the next task prompt |
| Skills/MCP | Run a task that needs a global skill and one MCP server | Skill is available; MCP is connected; the task uses the tool without a prompt |
| Security | Try `.env`, external path, subagent, destructive shell | Denied by generated agent permissions or policy gate |
| Restart | Kill and reopen the app | SQLite state survives; stale session error is recoverable |

Do not run the live matrix with provider keys until the user explicitly approves it.

## Manual verification checklist

- `Alt+\`` opens the 920x680 overlay in the real Tauri build.
- The overlay resizes down to 640x480 without clipping the composer.
- A multi-line draft grows the composer and leaves no internal scrollbar.
- The newest reply starts visible instead of scrolling past its first lines.
- Compact mode shows only the newest message; `Full thread · N` opens the rest.
- Project selector defaults to `CORE`.
- Chat mode sends a real prompt only in Tauri.
- Plan mode creates tasks and does not fabricate a reply.
- Task detail shows acceptance, dependencies, session id, and result.
- Running tasks can be aborted; review tasks can be accepted; blocked tasks can be retried.
- Dreamer button runs for the selected project; approval changes memory.
- Topbar says `Local data` in Tauri and `Browser preview` in the browser.
- Settings says local SQLite in Tauri and read-only preview in the browser.
- No secrets appear in the UI, logs, or generated agents.

Related: [[Home]] · [[Architecture]] · [[Data-Model]] · [[First-Draft]]
