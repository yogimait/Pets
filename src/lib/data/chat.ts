import type { ChatMessage } from "../types.ts";
import { minutesAgo } from "./clock.ts";

export const chat: ChatMessage[] = [
  {
    id: "chat-01",
    role: "user",
    text: "What is the workforce doing right now?",
    at: minutesAgo(26),
    refs: [],
  },
  {
    id: "chat-02",
    role: "orchestrator",
    text: "Three tasks are running. Pixel is finishing the orchestrator chat panel, Forge is on webhook retry ordering for Vault, and Probe is extending billing contract tests. Dreamer and Scout are idle, Voice is paused.",
    at: minutesAgo(25),
    refs: [
      { kind: "task", id: "t-102", label: "CORE t-102" },
      { kind: "task", id: "t-202", label: "VLT t-202" },
      { kind: "task", id: "t-207", label: "VLT t-207" },
    ],
  },
  {
    id: "chat-03",
    role: "user",
    text: "Anything blocked?",
    at: minutesAgo(18),
    refs: [],
  },
  {
    id: "chat-04",
    role: "orchestrator",
    text: "One task is blocked. Staging auth token rotation needs owner-issued credentials, so Probe cannot verify the access matrix. Everything else is queued or in review.",
    at: minutesAgo(17),
    refs: [
      { kind: "task", id: "t-205", label: "VLT t-205" },
      { kind: "agent", id: "probe", label: "Probe" },
    ],
  },
  {
    id: "chat-05",
    role: "user",
    text: "Plan the next step for Vault API.",
    at: minutesAgo(6),
    refs: [],
  },
  {
    id: "chat-06",
    role: "orchestrator",
    text: "The beta gate has three dependencies: webhook ordering must survive delayed delivery, idempotency keys must cover charge replays, and contract tests must include the entitlement endpoint. Proposed split: Forge finishes t-202 then takes idempotency keys, Probe finishes the entitlement contract tests, and Dreamer records the gate checklist. I would not start the load test until those three are green.",
    at: minutesAgo(5),
    refs: [
      { kind: "task", id: "t-202", label: "VLT t-202" },
      { kind: "task", id: "t-203", label: "VLT t-203" },
      { kind: "task", id: "t-207", label: "VLT t-207" },
    ],
  },
];
