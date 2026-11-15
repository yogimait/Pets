# Design-System

A developer command center: dark, matte, compact, quiet. Chrome carries structure; color carries status.

## Principles

1. Information before decoration. No gradients, glows, glass, or ornamental shadows.
2. One accent. Steel blue marks focus, active navigation, and primary action only.
3. Status colors are semantic. Green, amber, and red never act as branding.
4. Density with air. Rows are 32 to 40 px; panels sit on a 16 px rhythm.
5. Motion confirms a state change. Nothing moves to entertain.

## Tokens

Defined once in `src/app.css` under `@theme`.

| Token | Value | Use |
| --- | --- | --- |
| `canvas` | `#0a0b0d` | Window background |
| `surface` | `#111316` | Panels, sheets |
| `raised` | `#16181c` | Rows, chips, inputs |
| `hover` | `#1b1e23` | Hover and palette selection |
| `line` | `#23262b` | Strong borders |
| `line-soft` | `#1a1d21` | Default dividers |
| `ink` | `#e6e8eb` | Primary text |
| `ink-2` | `#9aa0a6` | Body text |
| `ink-3` | `#71777d` | Secondary metadata |
| `ink-4` | `#4d5359` | Disabled and hint text |
| `accent` | `#8ca6c0` | Focus, active nav, primary action |
| `accent-soft` | `#1a222b` | Active control fill |
| `ok` | `#6fa88a` | Active, running, done |
| `warn` | `#c8a05a` | Review, attention |
| `bad` | `#c07a7a` | Blocked, offline, critical |

Radii: `rounded-control` 6 px, `rounded-panel` 10 px. Base font size 13 px.

## Typography

- UI: Segoe UI Variable Text with Segoe UI and system fallbacks.
- IDs, timestamps, paths, model names, numeric badges: Cascadia Mono with system mono fallbacks.
- Headings: 18 px medium for view titles, 12 to 13 px medium for panel titles. No display sizes.
- Uppercase labels: 9.5 to 10 px, tracking 0.12 to 0.16 em, `ink-4`.

## Layout

- Sidebar 220 px, topbar 48 px, max content width 1120 px.
- Orchestrator overlay 920x680 default, resizable to a 640x480 floor, centered, frameless, on top of the desktop; no dock. Body text is 15px, controls are at least 32px high, and the composer grows with its draft up to 160px; longer drafts scroll inside the field so they cannot cover the thread.
- Pets are nine 56x80 transparent frameless always-on-top windows, visible at startup. Each agent has a distinct idle area and working area: idle pets wander slowly with pauses, active pets pace faster near the work area. Drag any pet to reposition it; the drop point becomes its new idle anchor. A shared 340x180 transparent comic speech-bubble window opens on hover/click; no app window appears.
- Detail sheets 460 px, full height, border-left, backdrop dim at 50%.
- Below 1200 px, secondary table columns drop before content truncates. Below 1280 px, overview panels collapse to one column.

## Components

| Component | Rule |
| --- | --- |
| Panel | 10 px radius, soft border, 40 px header, flush row content allowed |
| Status badge | 1.5 px dot plus label, tinted border, never a filled pill |
| Overview | Header, three-stat strip, active work, recent activity. No dashboard card grid. |
| Metrics | One divided strip, not four separate cards. |
| Avatar | Two-letter monogram, neutral raised square; accent border only for the orchestrator |
| Sheet | Focused close button, Tab trap, Escape closes, focus returns to trigger |
| Palette | 560 px panel, arrow navigation, accent left bar on the active row |
| Chat overlay | Black `#050505`, 1 px `#3d3d3d` border, zero radius, role labels in uppercase mono, no bubbles, green square only for the simulated marker |
| Assistant output | Safe Markdown subset: headings, paragraphs, lists, fenced code, inline code, bold, quotes, rules, and bordered tables. Raw HTML renders as text. Tables wrap; no horizontal page overflow. |
| Task form | Right sheet; project, owner, title, description, acceptance, priority, and start-now checkbox. Backend validates and returns the authoritative snapshot. |
| Pets | Nine independent 56x80 pixel sprites; transparent background; drag surface is exactly the sprite, not the transparent window margin; idle and working movement zones; shared comic speech bubble with name, role, status, current task, and latest activity. |

## Motion

- 140 to 160 ms, ease-out, opacity and transform only.
- Sheet slides 14 px; overlay fades; the working indicator pulses.
- `prefers-reduced-motion: reduce` collapses every animation and transition.
- No continuous animation or ambient movement. The layout is a snapshot, not a live feed.

## Content and honesty

- The desktop topbar carries `Local data`; the browser preview is labeled `Browser preview` and cannot change data.
- Chat shows `local` when the real workforce is available and `simulated` only in the browser preview. The overlay header and quiet keyboard hints stay visible; model reasoning is not shown, only the working state and the final reply.
- Compact mode shows only the newest message (the reply, or the prompt while working). The footer button opens the full thread and carries the hidden message count; there is no separate "show earlier" bar.
- The overlay is the one deliberate break from the charcoal system. It copies the daily tracker: black, square, monochrome.
- Empty or unconfigured states are explicit. The OpenRouter provider shows `Not configured` rather than hiding.
- The browser preview is read-only and cannot chat or run agents.

## Accessibility

- Visible focus ring on every interactive element, 1 px accent with 2 px offset.
- Sheets and palette use dialog semantics, accessible names, and keyboard dismissal.
- Tables keep header semantics; hidden columns are removed, not merely transparent.
- Contrast: body text on canvas is above 7:1; muted text is reserved for non-essential metadata.

Related: [[Home]] · [[Architecture]]
