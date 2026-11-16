# Agent Bots — project rules

Read this before changing the desktop app. Global rules in `~/.config/opencode/AGENTS.md` still apply.

## What this repo is

- `src/`, `src-tauri/`, `index.html`, `package.json`: Tauri 2 + Svelte 5 desktop interface.
- `.opencode/`, `tools/`, `taskboard.db`: existing OpenCode bot workspace, not wired to the app.
- `docs/`: Obsidian-style project documentation. Start at `docs/Home.md`.
- `taskboard.db`, `work/`: local runtime artifacts.

## Commands

```sh
npm run dev        # renderer at http://localhost:1420
npm run tauri dev  # desktop window
npm run check      # svelte-check, must stay at zero diagnostics
npm test           # dataset and selector invariants
npm run build      # production renderer
cargo check --manifest-path src-tauri/Cargo.toml
```

Port 1420 is strict. Stop a standalone `npm run dev` before `npm run tauri dev`, or the Tauri dev command fails on the occupied port.

This machine has limited free RAM. If a Tauri build fails with `only metadata stub found for rlib dependency core`, rerun with reduced jobs: `$env:CARGO_BUILD_JOBS=2; npm run tauri dev`.

## Rules

1. All workforce data enters through `src/lib/data/provider.ts`. Views and components must not import fixtures directly.
2. `src/lib/types.ts` is the contract. Keep it JSON-serializable and in epoch milliseconds.
3. New non-trivial selectors go in `src/lib/selectors.ts` with an invariant in `tests/data.test.ts`.
4. Use the design tokens in `src/app.css`. No new colors, shadows, gradients, or font families without updating `docs/Design-System.md`.
5. The desktop app has a real local backend: SQLite plus OpenCode sessions. The browser preview is read-only and cannot chat or run agents. Never claim a model ran, a task completed, or memory changed before the backend stored it.
6. The orchestrator is a separate resizable Tauri window (`orchestrator`, 920x680 default, 640x480 floor). Pets are nine 56x80 transparent always-on-top windows plus one shared comic speech-bubble window; do not turn them back into a docked strip. Pets are draggable and use role-specific idle and working movement. Topbar Pets toggles show/hide and pauses motion; closing the main window exits the app and all pets; a second launch focuses the existing instance. The workforce runs autonomously: queued tasks start, results auto-accept, and final project notifications fire. Windows login autostart keeps the loop available. Do not reintroduce a docked chat panel. The orchestrator visual language is the square black daily-tracker surface, not the charcoal panel system.
7. No dependency without a stated reason. The renderer uses Svelte, Tailwind, Lucide, and the Tauri API/plugins. Global shortcut, single-instance, notification, and autostart are the only Rust plugins.
8. Keep Tauri capabilities minimal. The current set is `core:default` plus `notification:default` for desktop task notifications. Autostart is registered from Rust through the autostart plugin.
9. Update the matching doc when architecture, data shape, tokens, or verification changes.
10. The global policy is read verbatim from `%USERPROFILE%\.config\opencode\AGENTS.md`. Do not copy it into prompts. Agent prompts live in `prompts/agents/`; keep each agent's `promptId` aligned with the prompt frontmatter.
11. OpenCode is the execution harness. The app is the control plane. Do not add a second model/tool loop or MCP client. Runtime calls belong in `src-tauri/src/opencode.rs`; the renderer talks only to app commands.
12. Task prompts carry the DevVault root, Home path, and project hub path; generated agents may access DevVault through the `external_directory` allow rule. Before committing meaningful work, update the project hub note, its Home row, and matching docs per global policy. Read-only/no-change tasks must not touch DevVault.

## Verification gate

Before declaring work done: `npm run check`, `npm test`, `npm run build`, and `cargo check` must pass. For UI work, also run the Impeccable detector and check the affected views at 1440x900 and 1080x720. For overlay work, verify the 920x680 window, resizing, and `Alt+\`` behavior in the real Tauri build.
