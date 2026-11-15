# Agent-Prompts

The workforce role definitions live in `prompts/`. They are original prompts informed by the parent/Dreamer separation in OpenAI Dots, not copied from leaked prompt text.

Reference: <https://github.com/asgeirtj/system_prompts_leaks/tree/main/OpenAI/dots>

## Precedence

1. Global `AGENTS.md` from `%USERPROFILE%\.config\opencode\AGENTS.md`, verbatim.
2. Project `AGENTS.md` from the selected project root.
3. Agent prompt in `prompts/agents/`.
4. Task context and approved project memory.
5. Shared handoff contract.

The higher layer wins. A project may extend or tighten the global rules, never replace them. An agent prompt may add role constraints, never relax policy. A real run requires both policy files.

## Files

| File | Role |
|---|---|
| `prompts/agents/orchestrator.md` | Planning, delegation, verification, reporting |
| `prompts/agents/dreamer.md` | Memory proposals, no actions |
| `prompts/agents/frontend.md` | Interface implementation, accessibility |
| `prompts/agents/backend.md` | APIs, data, migrations, auth, jobs |
| `prompts/agents/research.md` | Evidence, sources, comparisons |
| `prompts/agents/marketing.md` | Positioning, launch copy, claim review |
| `prompts/agents/ui-ux.md` | Flows, states, interaction specs |
| `prompts/agents/testing.md` | Verification and failure reproduction |
| `prompts/agents/auditing.md` | Security, permissions, data integrity, release risk |
| `prompts/shared/handoff.md` | Specialist return contract |
| `prompts/shared/memory-proposal.md` | Dreamer proposal contract |

## Agent sync

`scripts/sync-agents.mjs` converts the source prompts into OpenCode V2 primary agents:

| App agent | OpenCode id |
|---|---|
| Orchestrator | `orchestrator-workforce` |
| Dreamer | `dreamer` |
| Pixel | `pixel` |
| Forge | `forge` |
| Scout | `scout` |
| Voice | `voice` |
| Probe | `probe` |
| Iris | `iris` |
| Verity | `verity` |

Generated files are written to both `.opencode/agents/` and the global `%USERPROFILE%\.config\opencode\agent/`. The global copy is what makes the same app agents available in every selected project. Existing global files without the Agent Bots marker are not overwritten. They set:

- `mode: primary`
- the configured model, default `opencode-go/deepseek-v4.1-flash`; set `AGENT_BOTS_AGENT_MODEL` before sync to change it
- `steps: 24`
- permissions that allow MCP tools and deny subagents, questions, `.env` reads, external directories, and obvious destructive shell commands
- the original role body plus a small app runtime note for the orchestrator

Run `npm run sync:agents` after editing a prompt. Run `npm run check:agents` to validate.

## Runtime prompt assembly

For every task, Rust builds:

```text
global policy
project policy
role prompt from project `.opencode/agents/<agent>.md`, falling back to the global agent file
task id, title, description, acceptance
approved project memory, latest 8 entries
shared handoff contract
working directory reminder
```

The prompt is sent to an OpenCode session owned by the mapped agent. The agent's own OpenCode permissions still apply.

## Boundaries

- One owner per task.
- No recursive delegation.
- Maximum two concurrent specialists.
- Dreamer never contacts the user, executes tasks, or edits canonical memory.
- Specialists request help through the orchestrator/task board, not each other.
- Destructive, external, spending, publishing, or credential actions need approval.
- Verification is part of the deliverable.
- A simulated run must say it is simulated.

## Implemented

- Role prompts with frontmatter.
- Prompt parser and layered assembler.
- Policy reader for global and project `AGENTS.md`.
- Policy gate before every real run.
- Generated OpenCode agents.
- Agent sync and validation scripts.
- Task routing by owner id.
- Prompt preview in Agent detail.
- 27 Node tests and 17 Rust tests.

## Pending

- Streaming OpenCode events.
- Worktree isolation for coding tasks.
- Skill loading at runtime.
- Live model evaluation.
- Editable agent routing in the UI.

Related: [[Architecture]] · [[Testing]] · [[First-Draft]]
