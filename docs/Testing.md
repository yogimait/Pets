# Testing

## Commands

| Command | Purpose | Current result |
| --- | --- | --- |
| `npm run check` | Svelte and TypeScript diagnostics | 0 errors, 0 warnings |
| `npm test` | Dataset, selector, prompt, and runtime-format invariants | 22 tests pass |
| `npm run build` | Production renderer bundle | Passes, about 172 KB JS / 27 KB CSS |
| `cargo check --manifest-path src-tauri/Cargo.toml` | Rust shell type check | Passes |
| `npx tauri build` | Signed/bundled release | Not yet run in this phase |

## What the tests cover

`tests/data.test.ts` and `tests/prompts.test.ts` run on Node's built-in test runner with TypeScript type stripping, no framework:

1. Task owners and projects resolve.
2. Activity and memory references resolve, confidence is bounded.
3. Ids are unique per collection.
4. Current tasks are running and owned by the right agent.
5. Agent models resolve to configured providers.
6. Metrics match task statuses.
7. Project and step progress stay in range.
8. Activity and memory filters return subsets.
9. Chat reference chips point at real records.
10. The opening thread is internally consistent.
11. Proposals and Dreamer runs reference real records and carry evidence.
12. Every agent points at a registered prompt id.
13. Every expected role has exactly one prompt with valid frontmatter and a mission.
14. Prompts do not copy platform-specific tools or paths.
15. Prompt assembly preserves global, project, role, and handoff order.
16. Specialists receive the handoff contract; Dreamer receives the memory proposal contract.
17. The orchestrator routes every specialist.
18. Agent `promptId` values resolve.

The tests import fixtures, pure selectors, prompt functions, and runtime formatters directly from `src/`. No DOM, no Tauri, no network.

Runtime formatting tests cover user/assistant/tool normalization, malformed entries, permission normalization, and model key encoding.

## Manual verification

The renderer was exercised at 1440x900 and 1080x720 through Playwright. The desktop window ships at 1360x820 with a 1080x720 floor. Checks included:

- All seven views render with correct counts and empty states; the roster now has nine agents.
- Task, project, agent, and memory sheets open, scroll, and close.
- Esc closes sheets and palette; focus returns to the previous element.
- `Ctrl+K` opens the palette; typing `probe` and pressing Enter navigates to Agents and opens the Probe sheet.
- `Alt+\`` opens the 720x420 orchestrator overlay; it stays hidden between uses and the transcript survives close and reopen.
- Overlay sends on Enter, shows the pending line, returns a scripted reply, and toggles between the last exchange and the full thread.
- A reference chip hides the overlay and opens the linked task in the main window.
- Settings loads the global policy from `C:\Users\Hp\.config\opencode\AGENTS.md` with its byte count and hash; the project policy loads from the selected project root.
- Agent detail shows the assembled prompt with the global policy, project policy, role prompt, and the correct shared contract.
- A pending memory proposal opens, shows evidence and confidence, and can be approved or rejected for the session.
- Settings connects to a real OpenCode `2.1.3` service, lists built-in agents and tool-capable models, starts a real session, and shows the reply and tool calls.
- A shell permission request appears in the runtime panel; allowing it runs the command and shows the output.
- The simplified Overview shows a three-stat strip, active work, and recent activity without the previous card grid.
- Layout drops secondary table columns and collapses overview panels at 1080 px; the main content uses the full width with no chat dock.

Screenshots from that pass are in `work/` during development. They are not part of the app.

The Impeccable detector (`impeccable detect src --json`) reports zero findings on the current `src/` tree.

## Gaps

- No component-level or DOM test runner. The invariant harness covers data, not interaction.
- No visual regression baseline.
- No automated accessibility audit yet. Keyboard and focus behavior were checked manually.
- No automated Tauri IPC test. The app commands, global shortcut, and OpenCode runtime adapter were verified manually through WebView2 CDP.
- No test for the scripted responder beyond the invariant that its references resolve.
- Prompt assembly is tested in the renderer; the runtime does not yet block a run when the global policy is missing.
- Runtime message and permission updates use polling, not the OpenCode event stream.
- Runtime sessions are not linked to app tasks yet, and custom agents are not synced into OpenCode.

## Invariant to keep

Any change to fixtures or selectors must keep `npm test` green. Any change to a view must keep `npm run check` at zero diagnostics. A backend swap must keep the ten invariants valid against the returned snapshot.

Related: [[Home]] · [[Data-Model]] · [[Architecture]]
