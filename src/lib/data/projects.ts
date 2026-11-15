import type { Project } from "../types.ts";
import { daysAgo } from "./clock.ts";

export const projects: Project[] = [
  {
    id: "core",
    key: "CORE",
    name: "Agent Bots",
    description: "Desktop control center for the personal AI workforce.",
    status: "active",
    context:
      "Tauri 2 shell with a Svelte 5 interface. The desktop app stores workforce state in local SQLite and runs specialist work through OpenCode sessions. The browser-only preview keeps the fixture snapshot. Everything on screen must distinguish stored local state from simulated state.",
    instructions:
      "Keep the renderer thin and dependency-light. Route workforce reads and writes through the typed command client, never through fixtures. Prefer fewer, better screens over feature breadth.",
    repoPath: "D:\\Projects\\agent-bots",
    createdAt: daysAgo(21),
  },
  {
    id: "vault",
    key: "VLT",
    name: "Vault API",
    description: "Billing and entitlement service for the Atlas platform.",
    status: "active",
    context:
      "Node service on Postgres. Handles merchant subscriptions, entitlement checks, and webhook reconciliation for two payment providers. Beta is blocked until the billing contract and retry path are proven under load.",
    instructions:
      "Money paths require idempotency, an audit record, and a test. Webhook handlers must tolerate out-of-order delivery. No schema change ships without a forward-only migration.",
    repoPath: "D:\\Projects\\vault-api",
    createdAt: daysAgo(96),
  },
  {
    id: "field-notes",
    key: "FLD",
    name: "Field Notes",
    description: "Local inference cost research and benchmark memos.",
    status: "paused",
    context:
      "Internal research track. Compares local quantized models against hosted providers on cost, latency, and quality for coding tasks. Paused while the GPU box is rebuilt; measurements resume when the machine is back.",
    instructions:
      "Pin model revisions for every benchmark. Record tokens per second, memory footprint, and the exact quantization. Never generalize from a single run.",
    repoPath: "D:\\Projects\\field-notes",
    createdAt: daysAgo(54),
  },
  {
    id: "launch",
    key: "LCH",
    name: "Launch Page",
    description: "Marketing site for the Vault API beta.",
    status: "active",
    context:
      "Single-page site with pricing, waitlist, and three integration examples. Copy can only claim what the beta actually does. Waitlist form posts to the Vault API once staging credentials exist.",
    instructions:
      "Ship copy that matches the contract in vault/docs/Billing-API.md. No testimonials, no invented metrics. Keep the page under 120 KB of JavaScript.",
    repoPath: "D:\\Projects\\atlas-launch",
    createdAt: daysAgo(12),
  },
];
