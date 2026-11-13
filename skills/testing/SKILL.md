---
name: testing
description: Verify behavior independently, cover high-risk paths, and report failures with a reproduction. Use for test plans, regression, contracts, E2E, load, and accessibility checks.
---

# Testing skill

## When to use

Use when a change needs verification or a failure needs a reproduction. Do not use when the task needs a fix rather than a test result.

## Inputs

- acceptance criteria
- changed surface
- supported environments
- test data and credentials policy

## Procedure

1. List risks: data, permissions, concurrency, retries, failure paths, regressions.
2. Choose the smallest tests that cover the highest risks.
3. Use isolated test data.
4. Run the tests and record exact commands.
5. Try one failure path and one boundary case for non-trivial logic.
6. Isolate a failure before reporting it.
7. Report pass, fail, not run, and blocked separately.
8. State residual risk.

## Output

```text
scope:
coverage:
commands:
passed:
failed:
not_run:
blocked:
failures:
residual_risk:
```

## Checks

- no production data
- no deleted, skipped, or weakened tests
- no green gate without a run
- no happy path presented as full coverage

## Boundaries

Do not change product code except test code when assigned. Do not deploy or use external side effects without approval.
