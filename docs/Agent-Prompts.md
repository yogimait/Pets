# Agent Prompts

The workforce role definitions live in `prompts/`. They are original prompts for this project, informed by the parent/Dreamer separation in OpenAI Dots. They are not copied from the leaked prompt text.

Reference: <https://github.com/asgeirtj/system_prompts_leaks/tree/main/OpenAI/dots>

## Precedence

1. Global `AGENTS.md` from `%USERPROFILE%\.config\opencode\AGENTS.md`, verbatim.
2. Project `AGENTS.md` from the selected project root.
3. Agent prompt in `prompts/agents/`.
4. Task context and retrieved memory.
5. Shared handoff contract.

The higher layer wins. Project rules extend or tighten the global rules, never replace them. Agent prompts add role constraints, never relax policy. The harness records both policy hashes with every run.

## Files

| File | Role |
| --- | --- |
| `prompts/agents/orchestrator.md` | Parent, planning, delegation, verification, reporting |
| `prompts/agents/dreamer.md` | Memory research, proposals, no actions |
| `prompts/agents/frontend.md` | Interface implementation, accessibility, browser verification |
| `prompts/agents/backend.md` | APIs, data, migrations, auth, jobs, integrations |
| `prompts/agents/research.md` | Evidence, sources, comparisons, uncertainty |
| `prompts/agents/marketing.md` | Positioning, launch and landing copy, claim review |
| `prompts/agents/ui-ux.md` | Flows, IA, states, interaction specs, accessibility |
| `prompts/agents/testing.md` | Independent verification and failure reproduction |
| `prompts/agents/auditing.md` | Security, permissions, data integrity, release risk |
| `prompts/shared/handoff.md` | Specialist return contract |
| `prompts/shared/memory-proposal.md` | Dreamer proposal contract |

## Routing

| Request | First owner | Then |
| --- | --- | --- |
| New screen or flow | UI/UX | Frontend, Testing |
| Component or layout fix | Frontend | Testing |
| API, schema, auth, jobs | Backend | Testing, Auditing |
| Unknown facts or vendor choice | Research | Orchestrator decision |
| Launch copy or announcement | Marketing | Auditing for claims |
| Release check | Testing | Auditing |
| Memory consolidation | Dreamer | Orchestrator approval |

## Boundaries

- One owner per task.
- No recursive delegation.
- Maximum two concurrent specialists initially.
- Dreamer never contacts the user, executes tasks, or edits canonical memory.
- Specialists request help through the Orchestrator.
- Destructive, external, spending, publishing, or credential actions need approval.
- Verification is part of the deliverable.
- A simulated run must say it is simulated.

## Agent mapping

| App agent | Prompt id |
| --- | --- |
| `orchestrator` | `orchestrator` |
| `dreamer` | `dreamer` |
| `pixel` | `frontend` |
| `forge` | `backend` |
| `scout` | `research` |
| `voice` | `marketing` |
| `iris` | `ui-ux` |
| `probe` | `testing` |
| `verity` | `auditing` |

The app stores `promptId` on each agent, so a display name can change without breaking the prompt link.

## Status

Implemented:

- prompt files with frontmatter
- prompt parser and layered assembler
- bundled prompt loader
- Rust policy reader for global and project `AGENTS.md`
- policy path, hash, byte count, and reload control in Settings
- assembled prompt preview in Agent detail
- 18 invariant and prompt tests

Not implemented:

- runtime task routing
- hard block when the global policy is missing
- skill loading at runtime
- live provider calls
- persistence of policy snapshots

Related: [[Architecture]] · [[Design-System]] · [[Data-Model]] · [[Testing]]
