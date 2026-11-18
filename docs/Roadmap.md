# Roadmap

Progress tracker for the autonomy plan. The twelve issues come from the gap review on 2026-11-18; each is closed by the tier that owns it. Update this page in the same task that changes behavior.

## Where we are

**Tier 1 complete.** Image pipeline, chat actions, auto-retry, and session persistence shipped. Tier 2 is next.

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

## Tier 2 — Trustworthy output (next)

Closes issues 2, 5, 10.

- [ ] **2. Blind auto-accept** — verification gate: run the project's own checks before accepting a result; failure blocks the task with the check output.
- [ ] **5. No task streaming** — reuse the `session.text.*` delta bridge for task cards.
- [ ] **10. Memory injection is last-8** — rank approved memory by relevance to the task instead.

## Tier 3 — Run hot

Closes issues 7, 8, 9, 11.

- [ ] **7. One model for all roles** — per-role model selection in Settings (chat/plan/task/dreamer/vision).
- [ ] **8. Concurrency fixed at 2** — configurable 2-6 with a per-project cap.
- [ ] **9. No cost tracking** — per-task/project/day cost ledger from OpenCode session data with a soft ceiling.
- [ ] **11. Artifacts display-only** — store and browse agent-produced files and images.

## Tier 4 — Observable and scheduled

Closes issue 12.

- [ ] **12. No scheduled goals** — per-project cadence ("daily 9am: review yesterday, plan today") with a morning digest notification.
- [ ] Dreamer milestone trigger fires automatically at phase close (the scheduler hook, not a new loop).
- [ ] Merge-conflict recovery: retry with rebase onto a fresh base.

## Testing pass

After Tier 4: full regression — onboarding, chat+images+actions, plan, multi-phase autonomy, retry, restart mid-turn, dreamer with images, concurrency, security denials, installer.

## The twelve issues

| # | Issue | Tier | Status |
|---|---|---|---|
| 1 | Chat cannot act on work | 1 | done |
| 2 | Auto-accept without verification | 2 | open |
| 3 | Sessions lost on restart | 1 | done |
| 4 | Dreamer manual-only | 4 (hook) | open |
| 5 | No task streaming | 2 | open |
| 6 | No retry policy | 1 | done |
| 7 | One model for all roles | 3 | open |
| 8 | Concurrency fixed at 2 | 3 | open |
| 9 | No cost tracking | 3 | open |
| 10 | Memory last-8 not ranked | 2 | open |
| 11 | Artifacts display-only | 3 | open |
| 12 | No scheduled goals | 4 | open |

Related: [[Home]] · [[Architecture]] · [[Workflow]] · [[Testing]]
