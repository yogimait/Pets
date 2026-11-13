import type { ChatRef, Snapshot, Task } from "./types.ts";
import { agentById, buildMetrics } from "./selectors.ts";

export interface Reply {
  text: string;
  refs: ChatRef[];
}

export const SUGGESTIONS = [
  "What is running right now?",
  "Anything blocked?",
  "Plan the next step for Vault API",
  "What does Dreamer remember?",
];

function taskRef(snapshot: Snapshot, task: Task): ChatRef {
  const key = snapshot.projects.find((project) => project.id === task.projectId)?.key ?? task.projectId;
  return { kind: "task", id: task.id, label: `${key} ${task.id}` };
}

function agentRef(snapshot: Snapshot, id: string): ChatRef {
  return { kind: "agent", id, label: agentById(snapshot, id)?.name ?? id };
}

function describeTask(snapshot: Snapshot, task: Task): string {
  const owner = agentById(snapshot, task.ownerId)?.name ?? task.ownerId;
  return `${owner} — ${task.title} (${task.id})`;
}

export function respond(input: string, snapshot: Snapshot): Reply {
  const text = input.toLowerCase();
  const metrics = buildMetrics(snapshot);

  if (/(block|stuck|stall)/.test(text)) {
    const blocked = snapshot.tasks.filter((task) => task.status === "blocked");
    if (blocked.length === 0) {
      return { text: "Nothing is blocked right now. Every open task is queued, running, or in review.", refs: [] };
    }
    const lines = blocked.map((task) => `${task.title} (${task.id}) — ${task.notes || "no note recorded"}`);
    return {
      text: `${blocked.length} ${blocked.length === 1 ? "task is" : "tasks are"} blocked. ${lines.join(" ")} I will not retry these silently; each needs a decision or access from you.`,
      refs: blocked.flatMap((task) => [taskRef(snapshot, task), agentRef(snapshot, task.ownerId)]),
    };
  }

  if (/(vault|beta|next step|plan)/.test(text)) {
    const vault = snapshot.tasks.filter((task) => task.projectId === "vault");
    const running = vault.filter((task) => task.status === "running");
    const blocked = vault.filter((task) => task.status === "blocked");
    const queued = vault.filter((task) => task.status === "queued");
    const lines = [
      `Vault has ${vault.length} tasks: ${metrics.running > 0 ? `${running.length} running` : "none running"}, ${blocked.length} blocked, ${queued.length} queued.`,
      running.length > 0 ? `${running.map((task) => describeTask(snapshot, task)).join("; ")}.` : "",
      blocked.length > 0 ? `Gate: ${blocked.map((task) => task.title).join(", ")} needs your approval before verification can continue.` : "",
      queued.length > 0 ? `Next queued work: ${queued.slice(0, 2).map((task) => task.title).join(", ")}.` : "",
      "I would close webhook ordering, then idempotency keys, then the entitlement contract tests before starting the load run.",
    ];
    return {
      text: lines.filter(Boolean).join(" "),
      refs: [...running, ...blocked].slice(0, 4).map((task) => taskRef(snapshot, task)),
    };
  }

  if (/(memory|dreamer|remember|context)/.test(text)) {
    const latest = [...snapshot.memory].sort((a, b) => b.updatedAt - a.updatedAt).slice(0, 3);
    const projects = new Set(snapshot.memory.map((entry) => entry.projectId));
    return {
      text: `Dreamer holds ${snapshot.memory.length} entries across ${projects.size} projects. Most recent: ${latest.map((entry) => entry.title).join("; ")}. Confidence is tracked per entry, and everything is marked as simulated in this build.`,
      refs: [{ kind: "agent", id: "dreamer", label: "Dreamer" }],
    };
  }

  if (/(agent|team|who|workforce|running|status|now|doing)/.test(text)) {
    const active = snapshot.agents.filter((agent) => agent.status === "active");
    const lines = active.map((agent) => {
      const task = snapshot.tasks.find((candidate) => candidate.id === agent.currentTaskId);
      return task ? `${agent.name} on ${task.title} (${task.id})` : `${agent.name} is active without a claimed task`;
    });
    return {
      text: `${metrics.activeAgents} of ${metrics.totalAgents} agents are active. ${lines.join("; ")}. ${metrics.running} tasks are running, ${metrics.review} are in review, and ${metrics.blocked} is blocked.`,
      refs: active.map((agent) => agentRef(snapshot, agent.id)),
    };
  }

  return {
    text: "This panel is scripted in the current build. I can report running work, list blockers, outline the Vault beta path, or summarize what Dreamer remembers. No model is connected and nothing here executes work.",
    refs: [],
  };
}
