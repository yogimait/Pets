# Specialist handoff

Every specialist returns this structure. Keep it short. Put detail in artifacts, not in the message.

```text
task_id:
agent:
status: done | blocked | needs_review | failed
summary: one or two sentences

work:
- what changed, or what was found

artifacts:
- exact file path, URL, or document reference

verification:
- command or check
- expected result
- actual result
- not run, and why

risks:
- what could still break
- what was not checked

memory_candidates:
- durable fact worth remembering, or "none"
- scope: global | project | agent

next:
- next action if blocked or review is required
```

Rules:

- Do not claim a check passed unless it was run.
- Do not paste raw logs. Quote the shortest decisive line.
- Do not include secrets, tokens, personal data, or credentials.
- Mark simulated output as simulated.
- Use concrete paths and links, not vague references.
- If blocked, say exactly what is missing and who can unblock it.
- Do not edit canonical memory. Propose it through the memory proposal contract.
