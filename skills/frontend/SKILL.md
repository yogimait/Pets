---
name: frontend
description: Implement or review an interface change inside an existing frontend codebase. Use for components, layout, state, styling, accessibility, and browser verification.
---

# Frontend skill

## When to use

Use when the task changes a screen, component, interaction, style, or client state. Do not use for backend contracts, schema, or release work.

## Inputs

- acceptance criteria
- project design system and tokens
- existing components and patterns
- API contract for displayed data
- supported viewport widths and browsers

## Procedure

1. Read the project instructions and the affected code.
2. Find the existing pattern before writing a new component.
3. List states: loading, empty, error, success, disabled, permission denied.
4. Implement the smallest complete change.
5. Keep keyboard access, focus, labels, and reduced motion.
6. Run typecheck, lint, tests, and build.
7. Inspect the result at the minimum and a common desktop width.
8. Report files, checks, screenshots, and residual risk.

## Output

Use `prompts/shared/handoff.md`.

## Checks

- no dead controls
- no layout shift
- no color-only status
- no hidden focus
- no new dependency without reason
- no simulated data presented as live

## Boundaries

Do not change backend behavior, schema, deployment, or the design system outside the task.
