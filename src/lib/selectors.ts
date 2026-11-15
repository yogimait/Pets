import type {
  ActivityEvent,
  ActivityKind,
  Agent,
  ChatMessage,
  DreamerRun,
  MemoryEntry,
  MemoryKind,
  MemoryProposal,
  Project,
  Snapshot,
  Task,
  TaskStatus,
} from "./types.ts";

export function agentById(snapshot: Snapshot, id: string | null): Agent | undefined {
  return snapshot.agents.find((agent) => agent.id === id);
}

/** OpenCode agent name for real sessions. Falls back to the workforce agent id. */
export function opencodeAgentId(agent: Agent): string {
  return agent.opencodeAgentId ?? agent.id;
}

export function taskById(snapshot: Snapshot, id: string | null): Task | undefined {
  return snapshot.tasks.find((task) => task.id === id);
}

export function projectById(snapshot: Snapshot, id: string | null): Project | undefined {
  return snapshot.projects.find((project) => project.id === id);
}

export function memoryById(snapshot: Snapshot, id: string | null): MemoryEntry | undefined {
  return snapshot.memory.find((entry) => entry.id === id);
}

export function proposalById(snapshot: Snapshot, id: string | null): MemoryProposal | undefined {
  return snapshot.proposals.find((proposal) => proposal.id === id);
}

export function runById(snapshot: Snapshot, id: string | null): DreamerRun | undefined {
  return snapshot.dreamerRuns.find((run) => run.id === id);
}

export function pendingProposals(snapshot: Snapshot): MemoryProposal[] {
  return snapshot.proposals
    .filter((proposal) => proposal.status === "pending")
    .sort((a, b) => b.createdAt - a.createdAt);
}

export function proposalsForProject(snapshot: Snapshot, projectId: string): MemoryProposal[] {
  return snapshot.proposals
    .filter((proposal) => proposal.projectId === projectId)
    .sort((a, b) => b.createdAt - a.createdAt);
}

export function runsForProject(snapshot: Snapshot, projectId: string): DreamerRun[] {
  return snapshot.dreamerRuns
    .filter((run) => run.projectId === projectId)
    .sort((a, b) => b.startedAt - a.startedAt);
}

export function modelLabel(snapshot: Snapshot, agent: Agent): { provider: string; model: string } {
  for (const provider of snapshot.providers) {
    const model = provider.models.find((candidate) => candidate.id === agent.modelId);
    if (model && provider.id === agent.providerId) {
      return { provider: provider.name, model: model.name };
    }
  }
  return { provider: agent.providerId, model: agent.modelId };
}

export function tasksByProject(snapshot: Snapshot, projectId: string): Task[] {
  return snapshot.tasks.filter((task) => task.projectId === projectId).sort((a, b) => b.updatedAt - a.updatedAt);
}

export function chatForProject(
  snapshot: Snapshot,
  projectId: string,
  messages: ChatMessage[] = snapshot.chat,
): ChatMessage[] {
  return messages.filter((message) =>
    message.refs.some((reference) => {
      if (reference.kind === "project") return reference.id === projectId;
      if (reference.kind === "task") {
        return taskById(snapshot, reference.id)?.projectId === projectId;
      }
      return false;
    }),
  );
}

export function tasksByAgent(snapshot: Snapshot, agentId: string): Task[] {
  return snapshot.tasks.filter((task) => task.ownerId === agentId).sort((a, b) => b.updatedAt - a.updatedAt);
}

export function recentTasks(snapshot: Snapshot, agentId: string, limit = 3): Task[] {
  return tasksByAgent(snapshot, agentId)
    .filter((task) => task.status !== "done")
    .slice(0, limit);
}

export function taskStepProgress(task: Task): number {
  if (task.steps.length === 0) return 0;
  return task.steps.filter((step) => step.done).length / task.steps.length;
}

export function projectStats(snapshot: Snapshot, projectId: string): {
  total: number;
  done: number;
  running: number;
  progress: number;
} {
  const tasks = tasksByProject(snapshot, projectId);
  const done = tasks.filter((task) => task.status === "done").length;
  const running = tasks.filter((task) => task.status === "running").length;
  return { total: tasks.length, done, running, progress: tasks.length === 0 ? 0 : done / tasks.length };
}

export function buildMetrics(snapshot: Snapshot): {
  activeAgents: number;
  totalAgents: number;
  running: number;
  blocked: number;
  review: number;
  queued: number;
  done: number;
  memoryEntries: number;
  activeProjects: number;
} {
  const countByStatus = (status: TaskStatus) => snapshot.tasks.filter((task) => task.status === status).length;
  return {
    activeAgents: snapshot.agents.filter((agent) => agent.status === "active").length,
    totalAgents: snapshot.agents.length,
    running: countByStatus("running"),
    blocked: countByStatus("blocked"),
    review: countByStatus("review"),
    queued: countByStatus("queued"),
    done: countByStatus("done"),
    memoryEntries: snapshot.memory.length,
    activeProjects: snapshot.projects.filter((project) => project.status === "active").length,
  };
}

export function focusTasks(snapshot: Snapshot): Task[] {
  const rank: Record<TaskStatus, number> = { running: 0, blocked: 1, review: 2, queued: 3, done: 4 };
  return [...snapshot.tasks]
    .filter((task) => task.status !== "done")
    .sort((a, b) => rank[a.status] - rank[b.status] || b.updatedAt - a.updatedAt);
}

export interface ActivityFilter {
  agentId?: string | null;
  projectId?: string | null;
  kind?: ActivityKind | "all";
}

export function activityFiltered(snapshot: Snapshot, filter: ActivityFilter = {}): ActivityEvent[] {
  return snapshot.activity.filter((event) => {
    if (filter.agentId && event.actorId !== filter.agentId) return false;
    if (filter.projectId && event.projectId !== filter.projectId) return false;
    if (filter.kind && filter.kind !== "all" && event.kind !== filter.kind) return false;
    return true;
  });
}

export function activityByDay(events: ActivityEvent[]): { label: string; events: ActivityEvent[] }[] {
  const groups: { label: string; events: ActivityEvent[] }[] = [];
  for (const event of events) {
    const label = new Date(event.at).toLocaleDateString(undefined, {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
    const last = groups.at(-1);
    if (last && last.label === label) last.events.push(event);
    else groups.push({ label, events: [event] });
  }
  return groups;
}

export interface MemoryFilter {
  projectId?: string | null;
  kind?: MemoryKind | "all";
  query?: string;
}

export function memoryFiltered(snapshot: Snapshot, filter: MemoryFilter = {}): MemoryEntry[] {
  const query = filter.query?.trim().toLowerCase() ?? "";
  return snapshot.memory
    .filter((entry) => {
      if (filter.projectId && entry.projectId !== filter.projectId) return false;
      if (filter.kind && filter.kind !== "all" && entry.kind !== filter.kind) return false;
      if (!query) return true;
      return [entry.title, entry.content, ...entry.tags].join(" ").toLowerCase().includes(query);
    })
    .sort((a, b) => b.updatedAt - a.updatedAt);
}

export function artifactsFor(snapshot: Snapshot, projectId: string) {
  return snapshot.artifacts.filter((artifact) => artifact.projectId === projectId);
}

export function agentsForProject(snapshot: Snapshot, projectId: string): Agent[] {
  return snapshot.agents.filter((agent) => agent.projectIds.includes(projectId));
}

export function providerFor(snapshot: Snapshot, agent: Agent) {
  return snapshot.providers.find((provider) => provider.id === agent.providerId);
}
