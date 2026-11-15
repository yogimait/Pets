import { invoke, isTauri } from "@tauri-apps/api/core";
import type { ProposalStatus, Snapshot, Task } from "./types.ts";

// Typed wrappers for the Rust workforce commands. The renderer never owns state;
// every call returns the authoritative snapshot the backend stored.

export interface PlanResult {
  summary: string;
  tasks: Task[];
}

export interface ProjectInspection {
  path: string;
  exists: boolean;
  isDirectory: boolean;
  hasAgents: boolean;
  hasReadme: boolean;
  isGit: boolean;
  entryCount: number;
}

export interface TaskInput {
  projectId: string;
  title: string;
  description: string;
  ownerId: string;
  priority: "low" | "normal" | "high" | "critical";
  acceptance: string;
  autoStart: boolean;
}

export interface ProjectInput {
  name: string;
  key: string;
  repoPath: string;
  description: string;
  context: string;
  instructions: string;
  createFolder: boolean;
  createPolicy: boolean;
  createReadme: boolean;
  initGit: boolean;
}

export function workforceAvailable(): boolean {
  return isTauri();
}

export function workforceSnapshot(): Promise<Snapshot | null> {
  return invoke<Snapshot | null>("workforce_snapshot");
}

export function workforceSeed(snapshot: Snapshot): Promise<Snapshot> {
  return invoke<Snapshot>("workforce_seed", { snapshot });
}

export function workforceInspectProject(path: string): Promise<ProjectInspection> {
  return invoke<ProjectInspection>("workforce_inspect_project", { path });
}

export function workforceCreateProject(input: ProjectInput): Promise<Snapshot> {
  return invoke<Snapshot>("workforce_create_project", {
    name: input.name,
    key: input.key,
    repoPath: input.repoPath,
    description: input.description,
    context: input.context,
    instructions: input.instructions,
    createFolder: input.createFolder,
    createPolicy: input.createPolicy,
    createReadme: input.createReadme,
    initGit: input.initGit,
  });
}

export function workforceCreateTask(input: TaskInput): Promise<Snapshot> {
  return invoke<Snapshot>("workforce_create_task", {
    projectId: input.projectId,
    title: input.title,
    description: input.description,
    ownerId: input.ownerId,
    priority: input.priority,
    acceptance: input.acceptance,
    autoStart: input.autoStart,
  });
}

export function workforcePlan(projectId: string, goal: string): Promise<PlanResult> {
  return invoke<PlanResult>("workforce_plan", { projectId, goal });
}

export function workforceSend(projectId: string, text: string): Promise<{ text: string }> {
  return invoke<{ text: string }>("workforce_send", { projectId, text });
}

export function workforceTick(): Promise<Snapshot> {
  return invoke<Snapshot>("workforce_tick");
}

export function workforcePoll(): Promise<Snapshot> {
  return invoke<Snapshot>("workforce_poll");
}

export type PermissionDecision = "once" | "always" | "reject";

export function workforceReplyTaskPermission(
  taskId: string,
  requestId: string,
  decision: PermissionDecision,
): Promise<Snapshot> {
  return invoke<Snapshot>("workforce_reply_task_permission", { taskId, requestId, decision });
}

export function workforceRunTask(taskId: string): Promise<Snapshot> {
  return invoke<Snapshot>("workforce_run_task", { taskId });
}

export function workforceAcceptTask(taskId: string): Promise<Snapshot> {
  return invoke<Snapshot>("workforce_accept_task", { taskId });
}

export function workforceAbortTask(taskId: string): Promise<Snapshot> {
  return invoke<Snapshot>("workforce_abort_task", { taskId });
}

export function workforceDreamer(projectId: string): Promise<Snapshot> {
  return invoke<Snapshot>("workforce_dreamer", { projectId });
}

export function workforceDecideProposal(
  proposalId: string,
  decision: Extract<ProposalStatus, "approved" | "rejected">,
): Promise<Snapshot> {
  return invoke<Snapshot>("workforce_decide_proposal", { proposalId, decision });
}
