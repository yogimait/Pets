# Agent prompts

Original role prompts for the Agent Bots workforce. They are informed by the architecture of OpenAI Dots, not copied from the leaked prompt text.

Reference: <https://github.com/asgeirtj/system_prompts_leaks/tree/main/OpenAI/dots>

## Policy precedence

The harness loads policy before these files:

1. Global `AGENTS.md` from `%USERPROFILE%\.config\opencode\AGENTS.md`, verbatim.
2. Project `AGENTS.md` from the selected project root.
3. The agent prompt in `prompts/agents/`.
4. Task context and retrieved memory.
5. The shared handoff contract.

If two layers conflict, the higher layer wins. A project may extend or tighten the global rules, never replace them. An agent prompt may add role constraints, never relax policy.

The harness records the global and project policy hashes with every run. If a policy file is missing, the run does not start. Do not copy the global rules into these files; one canonical source only.

Some global rules reference tools that a given agent may not have. The text stays unchanged; an unavailable tool is inert. An agent must never claim it used a tool it does not have.

## What we borrowed from Dots

- Parent and Dreamer are separate agents with different objectives.
- Dreamer researches and proposes memory changes. It never contacts the user, never acts, and never edits canonical memory.
- Memory is layered: global, project, agent, task.
- Dreamer reports to the parent; the parent decides.
- Run logs, sources, confidence, and proposals matter more than prose.
- Specialist skills are separate packages, loaded only when relevant.
- Background work is triggered, not continuous.

We did not copy platform tool names, cloud paths, Heartbeat text, skill content, or the parent prompt, which is not present in the repository.

## Layout

```text
prompts/
  README.md
  shared/
    handoff.md
    memory-proposal.md
  agents/
    orchestrator.md
    dreamer.md
    frontend.md
    backend.md
    research.md
    marketing.md
    ui-ux.md
    testing.md
    auditing.md
```

## Design rules

- One owner per task.
- One deliverable per task.
- No recursive delegation.
- Maximum two concurrent specialists initially.
- Every task has acceptance criteria before it starts.
- Verification is part of the deliverable.
- Destructive, external, or spending actions need approval.
- Memory writes are proposals until approved.
- A simulated run must say it is simulated.

## Adding an agent

1. Add a prompt with frontmatter.
2. State what it owns, what it does not own, and its output contract.
3. Add its skill package under `skills/<name>/SKILL.md` when tools or procedures are needed.
4. Add it to the Orchestrator routing table.
5. Add a routing and boundary test in the harness.
