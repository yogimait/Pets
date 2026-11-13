---
name: ui-ux
description: Specify a flow, screen structure, interaction, or state model before implementation. Use for information architecture, wireframes, accessibility, and design-system decisions.
---

# UI/UX skill

## When to use

Use when the user goal is clear but the flow, layout, states, or interaction are not. Do not use to implement production code unless the task asks for a prototype.

## Inputs

- user goal and success condition
- existing design system and components
- technical constraints
- available research, or an explicit assumption

## Procedure

1. State the user, job, and success condition.
2. Map the shortest flow.
3. Identify entry, exit, cancel, and failure paths.
4. Choose layout and hierarchy using existing patterns.
5. Specify loading, empty, error, success, disabled, permission, and offline states.
6. Specify keyboard, focus, hover, escape, cancel, and undo.
7. Write labels, helper text, errors, and confirmations.
8. Check responsive behavior and accessibility.
9. Write acceptance criteria for the frontend agent.

## Output

```text
goal:
user:
success:
flow:
screens:
states:
interaction:
content:
accessibility:
acceptance:
open_questions:
```

## Checks

- one primary action per view
- destructive actions distinct and confirmed
- no essential information only on hover
- error says what happened and how to recover
- no color-only meaning
- reduced motion respected

## Boundaries

Do not invent research, add a new visual language, expand scope, or claim usability testing that did not happen.
