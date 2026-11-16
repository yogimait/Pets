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
| Micheal · Orchestrator | `orchestrator-workforce` |
| Holly · Memory | `dreamer` |
| Jim · Frontend | `pixel` |
| Dwight · Backend | `forge` |
| Ryan · Research | `scout` |
| Andy · Marketing | `voice` |
| Erin · Testing | `probe` |
| Pam · UI/UX | `iris` |
| Stanley · Auditing | `verity` |

Generated files are written to both `.opencode/agents/` and the global `%USERPROFILE%\.config\opencode\agent/`. The global copy is what makes the same app agents available in every selected project. The release binary embeds these files and installs missing or Agent-Bots-generated copies on startup. Existing global files without the Agent Bots marker are not overwritten. They set:

- `mode: primary`
- the configured model, default `opencode-go/deepseek-v4.1-flash`; set `AGENT_BOTS_AGENT_MODEL` before sync to change it
- `steps: 24`
- permissions that allow workspace tools and MCP in specialist sessions, and deny subagents, questions, `.env` reads, other external directories, and obvious destructive shell commands
- the orchestrator can read files and run non-destructive shell commands; it cannot edit files or accept tasks
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
DevVault root, Home path, project hub path, and maintenance rule
shared handoff contract
working directory reminder
```

The prompt is sent to an OpenCode session owned by the mapped agent. The agent's own OpenCode permissions still apply.

## Code quality gate

The frontend, backend, and task prompts now require an explicit component/module structure, runnable typecheck/tests/build, and a final independent quality/audit gate for implementation work. The auditing role reviews structure, duplication, dead code, testability, and convention drift in addition to security.

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
- 28 Node tests and 26 Rust tests.

## Pending

- Skill loading at runtime.
- Editable agent routing in the UI.
- Manual task creation outside Plan.

Related: [[Architecture]] Â· [[Testing]] Â· [[First-Draft]]
