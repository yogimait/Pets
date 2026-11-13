export type ProviderKind = "cloud" | "local";
export type AgentStatus = "active" | "idle" | "paused" | "offline";
export type ProjectStatus = "active" | "paused" | "archived";
export type TaskStatus = "queued" | "running" | "review" | "blocked" | "done";
export type TaskPriority = "low" | "normal" | "high" | "critical";
export type ActivityKind = "delegation" | "status" | "message" | "result" | "memory" | "system";
export type MemoryKind = "decision" | "reference" | "doc" | "result" | "convention";
export type ArtifactKind = "file" | "url" | "doc";

export interface Model {
  id: string;
  name: string;
  context: number;
  note: string;
}

export interface Provider {
  id: string;
  name: string;
  kind: ProviderKind;
  endpoint: string;
  configured: boolean;
  models: Model[];
}

export interface Agent {
  id: string;
  name: string;
  role: string;
  description: string;
  status: AgentStatus;
  providerId: string;
  modelId: string;
  promptId: string;
  capabilities: string[];
  instructions: string;
  projectIds: string[];
  currentTaskId: string | null;
  successRate: number;
  tasksCompleted: number;
  lastActiveAt: number;
}

export interface Project {
  id: string;
  key: string;
  name: string;
  description: string;
  status: ProjectStatus;
  context: string;
  instructions: string;
  repoPath: string;
  createdAt: number;
}

export interface Artifact {
  id: string;
  projectId: string;
  name: string;
  kind: ArtifactKind;
  path: string;
}

export interface TaskMessage {
  id: string;
  senderId: string;
  text: string;
  at: number;
}

export interface TaskStep {
  label: string;
  done: boolean;
}

export interface Task {
  id: string;
  title: string;
  description: string;
  status: TaskStatus;
  priority: TaskPriority;
  projectId: string;
  ownerId: string;
  notes: string;
  tags: string[];
  createdAt: number;
  updatedAt: number;
  steps: TaskStep[];
  messages: TaskMessage[];
}

export interface ActivityEvent {
  id: string;
  at: number;
  kind: ActivityKind;
  actorId: string;
  projectId: string | null;
  taskId: string | null;
  summary: string;
  detail?: string;
}

export interface MemoryEntry {
  id: string;
  title: string;
  kind: MemoryKind;
  projectId: string;
  content: string;
  tags: string[];
  source: string;
  confidence: number;
  createdAt: number;
  updatedAt: number;
  linkedTaskId: string | null;
}

export interface ChatRef {
  kind: "task" | "agent" | "project";
  id: string;
  label: string;
}

export interface ChatMessage {
  id: string;
  role: "user" | "orchestrator";
  text: string;
  at: number;
  refs: ChatRef[];
}

export type ProposalAction = "add" | "update" | "delete" | "merge";
export type ProposalStatus = "pending" | "approved" | "rejected";

export interface MemoryProposal {
  id: string;
  action: ProposalAction;
  scope: "global" | "project" | "agent";
  projectId: string;
  agentId: string | null;
  targetEntryId: string | null;
  kind: MemoryKind;
  title: string;
  content: string;
  reason: string;
  evidence: string[];
  confidence: number;
  conflictsWith: string | null;
  status: ProposalStatus;
  createdAt: number;
  decidedAt: number | null;
}

export type DreamerTrigger = "manual" | "milestone" | "reflection";

export interface DreamerRun {
  id: string;
  trigger: DreamerTrigger;
  scope: string;
  projectId: string;
  startedAt: number;
  finishedAt: number | null;
  sourcesRead: string[];
  sourcesSkipped: string[];
  findings: string[];
  proposalIds: string[];
  status: "running" | "done" | "partial" | "failed";
  parentUpdate: "none" | "sent";
}

export interface Snapshot {
  agents: Agent[];
  projects: Project[];
  tasks: Task[];
  activity: ActivityEvent[];
  memory: MemoryEntry[];
  proposals: MemoryProposal[];
  dreamerRuns: DreamerRun[];
  providers: Provider[];
  artifacts: Artifact[];
  chat: ChatMessage[];
}
