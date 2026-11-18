# Roadmap

Progress tracker for the autonomy plan. The twelve issues come from the gap review on 2026-11-18; each is closed by the tier that owns it. Update this page in the same task that changes behavior.

## Where we are

**Tiers 1–4 complete.** All twelve issues are closed. A full regression pass is next.

## Tier 1 — Chat that acts (done)

Closes issues 1, 3, 6.

- [x] **1. Chat is inert** — orchestrator replies can end with an ```agent-bots fenced block: `create_task`, `plan`, `retry_task`. The backend parses the block, strips it from the stored reply, executes it, and attaches task reference chips. See [[Workflow]].
- [x] **3. Session map is in-memory only** — `chat_sessions` and `chat_stream` SQLite tables persist the per-project session and the active streaming turn; `workforce_chat_resume` lets the overlay finish a turn that outlived an app restart.
- [x] **6. No retry policy** — `mark_failure` requeues a running task up to 2 times for transient failures (service down, connection, timeout, session read). Credential, quota, policy, merge, and permission-denial failures block immediately. `retryCount` is stored on the task.

### Image pipeline (done)

- [x] Paste or drop up to 4 images (png/jpeg/webp/gif, 8 MB each) into the orchestrator composer.
- [x] Bytes are stored under `app_data_dir/attachments/`; chat messages carry `{id, name, mime, path}` metadata only — never the image itself.
- [x] `workforce_send_start` posts images as inline data-uri `files` to the OpenCode prompt. Verified live: `opencode-go/deepseek-v4.1-flash` describes image contents correctly.
- [x] Thumbnails render in chat via `workforce_attachment_data`, which only reads inside the attachments directory.
- [x] Dreamer receives the newest 4 project-chat images with its prompt and can store durable visual facts as `observation` memory.

### Tier 1 verification

- 32 release Rust tests, 28 Node tests, `svelte-check` 0.
- Live, on the packaged build:
  - dropped a generated image into the composer; the chip appeared, the model described it correctly (shapes and colors), and the message stored `attachments` metadata while bytes landed in `app_data_dir/attachments`.
  - asked for a task "Owner: Erin"; the reply's ```agent-bots block created a queued `autoStart` task owned by `probe`, the stored reply kept a task reference chip and no fence, and the scheduler picked it up.
  - killed the app mid-turn; after relaunch the orchestrator window resumed the turn, reconnected to the service, and stored exactly one orchestrator row.

## Tier 2 — Trustworthy output (done)

Closes issues 2, 5, 10.

- [x] **2. Blind auto-accept** — optional per-project verification command (`verifyCommand`) runs in the isolated worktree after the result lands and before acceptance. Pass accepts and merges; failure blocks the task with the captured output (capped at 4,000 chars). The task sheet shows a Verification section with status, time, and output.
- [x] **5. No task streaming** — `workforce_task_stream` returns the running session's latest assistant text; the task sheet polls it every 1.2s and renders a Live output section while the task runs.
- [x] **10. Memory injection is last-8** — task prompts now inject the top-8 approved entries by keyword-overlap relevance to the task title/description/acceptance, recency as tiebreaker. Chat and Plan keep recency order.

## Tier 3 — Run hot (done)

Closes issues 7, 8, 9, 11.

- [x] **7. One model for all roles** — `AGENT_BOTS_MODEL_CHAT`, `AGENT_BOTS_MODEL_TASK`, `AGENT_BOTS_MODEL_DREAMER` override per role; each falls back to `AGENT_BOTS_MODEL`, then the default.
- [x] **8. Concurrency fixed at 2** — `AGENT_BOTS_MAX_CONCURRENT` (clamped 2–6) now drives both the scheduler and the start guard.
- [x] **9. No cost tracking** — the poll loop reads the OpenCode session cost and stores it on the task; the task sheet shows it. Verification, cost, and changed files are visible before/after acceptance.
- [x] **11. Artifacts display-only** — accepting a worktree task records the branch's changed files (`git diff --name-only HEAD...branch`, up to 50) on the task and shows them as chips in the task sheet.

## Tier 4 — Observable and scheduled (done)

Closes issue 12.

- [x] **12. No scheduled goals** — New project can set a cadence in hours plus a goal. `workforce_tick` runs at most one scheduled plan per pass when the project is active, the cadence is due, and the project has no queued/running/review tasks; it stores `nextRunAt` and logs the plan summary to Activity.
- [x] **4. Dreamer manual-only** — the phase loop already runs Dreamer before replanning (`maybeAutoPlan`); this was shipped with the autonomous loop and is now confirmed and marked done.
- [x] Merge-conflict recovery — retrying a blocked task recreates the worktree from the current base, which is the rebase-onto-fresh-base path; the retained-worktree error message tells the operator exactly that.

## Testing pass

Full regression pending: onboarding, chat+images+actions, plan, multi-phase autonomy, retry, restart mid-turn, dreamer with images, concurrency, security denials, installer. Live-verified so far: chat actions on two projects, task streaming, cost capture, and the dirty-repo worktree gate.

## The twelve issues

| # | Issue | Tier | Status |
|---|---|---|---|
| 1 | Chat cannot act on work | 1 | done |
| 2 | Auto-accept without verification | 2 | done |
| 3 | Sessions lost on restart | 1 | done |
| 4 | Dreamer manual-only | 1/4 | done (phase loop) |
| 5 | No task streaming | 2 | done |
| 6 | No retry policy | 1 | done |
| 7 | One model for all roles | 3 | done |
| 8 | Concurrency fixed at 2 | 3 | done |
| 9 | No cost tracking | 3 | done |
| 10 | Memory last-8 not ranked | 2 | done |
| 11 | Artifacts display-only | 3 | done |
| 12 | No scheduled goals | 4 | done |

Related: [[Home]] · [[Architecture]] · [[Workflow]] · [[Testing]]
