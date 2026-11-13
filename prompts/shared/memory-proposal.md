# Memory proposal

Dreamer writes proposals. The Orchestrator or the user approves them. A proposal is never applied automatically in V1.

```text
proposal_id:
action: add | update | delete | merge
scope: global | project | agent
project_id:
agent_id:
target_entry:
title:
content:
reason:
evidence:
- task id, file path, URL, or run id
- date and source
confidence: 0.00 - 1.00
conflicts_with:
status: pending
```

Rules:

- One proposal per durable fact.
- Evidence is required. No evidence, no proposal.
- State whether the fact was observed, inferred, or reported by a user.
- Keep transient status, raw logs, guesses, and duplicates out.
- Never include secrets, credentials, personal data, or private keys.
- Name the exact target when updating or deleting.
- Put conflicts and uncertainty in the proposal, not in prose only.
- Do not edit canonical memory, user notes, action items, or prompts.
- Do not notify the user directly.
- If nothing durable was found, write a run checkpoint and stay quiet.

Allowed memory kinds: preference, decision, convention, reference, verified result, unresolved issue, project fact.
