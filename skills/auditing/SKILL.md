---
name: auditing
description: Independently review security, permissions, data integrity, dependencies, policy compliance, and release readiness. Read-only by default.
---

# Auditing skill

## When to use

Use before a release or when a change touches money, auth, permissions, data loss, secrets, dependencies, or external actions. Testing verifies expected behavior; auditing challenges risk.

## Inputs

- task and acceptance criteria
- changed surface
- policy files
- deployment and data assumptions

## Procedure

1. Map trust boundaries and data flows.
2. Identify assets: credentials, personal data, money, destructive actions, external side effects.
3. Check the highest-impact failure first.
4. Inspect auth, validation, migrations, logging, secrets, dependencies, and prompt injection.
5. Collect evidence for every finding.
6. Separate confirmed findings from questions.
7. Rate severity by impact and likelihood.
8. Give a verdict and minimum required actions.

## Output

```text
scope:
verdict:
findings:
  - id
  - severity
  - title
  - evidence
  - impact
  - fix
questions:
not_checked:
residual_risk:
required_actions:
```

## Checks

- no secret in repo, logs, prompts, or client bundle
- authorization enforced server-side
- migrations have a recovery path
- logs do not contain unnecessary personal data
- dependencies and licenses reviewed
- retrieved content cannot redefine instructions

## Boundaries

Read-only by default. Do not fix, merge, deploy, publish, or approve on the user's behalf. Do not use production data or cause side effects.
