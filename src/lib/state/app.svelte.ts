import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ChatMessage, ProposalStatus, Snapshot } from "../types.ts";
import { loadSnapshot } from "../data/provider.ts";
import { readPolicy, type PolicyBundle } from "../policy.ts";
import {
  workforceAcceptTask,
  workforceAbortTask,
  workforceAvailable,
  workforceCreateProject,
  workforceCreateTask,
  workforceDecideProposal,
  workforceDreamer,
  workforcePlan,
  workforcePoll,
  workforceReplyTaskPermission,
  workforceRunTask,
  workforceSend,
  workforceSnapshot,
  workforceTick,
  type PermissionDecision,
  type PlanResult,
  type ProjectInput,
  type TaskInput,
} from "../workforce.ts";
import type { View } from "../nav.ts";

export type SheetKind = "agent" | "project" | "project-new" | "task" | "memory" | "proposal";

export interface SheetRef {
  kind: SheetKind;
  id: string;
}

export const app = $state({
  view: "overview" as View,
  ready: false,
  snapshot: null as Snapshot | null,
  paletteOpen: false,
  overlayOpen: false,
  petsVisible: true,
  sheet: null as SheetRef | null,
  chat: [] as ChatMessage[],
  pending: false,
  policy: null as PolicyBundle | null,
  policyProjectId: null as string | null,
  policyLoading: false,
  policyError: null as string | null,
  workforceError: null as string | null,
  workforceBusy: false,
});

let started = false;

async function loadInitialSnapshot(): Promise<Snapshot> {
  for (let attempt = 0; ; attempt++) {
    try {
      return await loadSnapshot();
    } catch (error) {
      const message = String(error);
      if (attempt >= 20 || !message.includes("state not managed")) throw error;
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
  }
}

export async function init(): Promise<void> {
  if (started) return;
  started = true;
  try {
    const snapshot = await loadInitialSnapshot();
    app.snapshot = snapshot;
    app.chat = snapshot.chat ?? [];
  } catch (error) {
    app.workforceError = String(error);
    return;
  }
  await loadPolicy("core");
  app.ready = true;
  startWorkforceSync();
}

export async function loadPolicy(projectId: string | null): Promise<void> {
  const project = projectId ? app.snapshot?.projects.find((candidate) => candidate.id === projectId) : undefined;
  app.policyProjectId = projectId;
  app.policyLoading = true;
  app.policyError = null;
  try {
    app.policy = await readPolicy(project?.repoPath ?? null);
  } catch (error) {
    app.policy = null;
    app.policyError = String(error);
  } finally {
    app.policyLoading = false;
  }
}

export async function reloadPolicy(): Promise<void> {
  await loadPolicy(app.policyProjectId);
}

// Every workforce action returns the authoritative snapshot. View and sheet stay untouched.
async function runWorkforceAction(call: () => Promise<Snapshot>): Promise<boolean> {
  app.workforceBusy = true;
  app.workforceError = null;
  try {
    app.snapshot = await call();
    return true;
  } catch (error) {
    app.workforceError = String(error);
    return false;
  } finally {
    app.workforceBusy = false;
  }
}

export async function refreshWorkforce(): Promise<void> {
  if (!workforceAvailable()) return;
  try {
    const next = await workforceSnapshot();
    if (!next) return;
    app.snapshot = next;
    app.chat = next.chat ?? [];
    app.workforceError = null;
  } catch (error) {
    app.workforceError = String(error);
  }
}

export async function createProject(input: ProjectInput): Promise<string | null> {
  app.workforceBusy = true;
  app.workforceError = null;
  try {
    const next = await workforceCreateProject(input);
    app.snapshot = next;
    const wantedKey = input.key.trim().toUpperCase();
    return next.projects.find((project) => project.key.toUpperCase() === wantedKey)?.id ?? null;
  } catch (error) {
    app.workforceError = String(error);
    return null;
  } finally {
    app.workforceBusy = false;
  }
}

export function createTask(input: TaskInput): Promise<boolean> {
  return runWorkforceAction(() => workforceCreateTask(input));
}

export async function sendOrchestrator(projectId: string, text: string): Promise<boolean> {
  app.workforceError = null;
  try {
    const reply = await workforceSend(projectId, text);
    const project = app.snapshot?.projects.find((candidate) => candidate.id === projectId);
    app.chat.push({
      id: `chat-${Date.now()}-r`,
      role: "orchestrator",
      text: reply.text,
      at: Date.now(),
      refs: [{ kind: "project", id: projectId, label: project?.key ?? projectId }],
    });
    return true;
  } catch (error) {
    app.workforceError = String(error);
    return false;
  }
}

export async function planWork(projectId: string, goal: string): Promise<PlanResult | null> {
  app.workforceBusy = true;
  app.workforceError = null;
  try {
    const result = await workforcePlan(projectId, goal);
    await refreshWorkforce();
    return result;
  } catch (error) {
    app.workforceError = String(error);
    return null;
  } finally {
    app.workforceBusy = false;
  }
}

export function runTask(taskId: string): Promise<boolean> {
  return runWorkforceAction(() => workforceRunTask(taskId));
}

export function replyTaskPermission(
  taskId: string,
  requestId: string,
  decision: PermissionDecision,
): Promise<boolean> {
  return runWorkforceAction(() => workforceReplyTaskPermission(taskId, requestId, decision));
}

export async function acceptTask(taskId: string): Promise<boolean> {
  const accepted = await runWorkforceAction(() => workforceAcceptTask(taskId));
  if (accepted) void syncNow();
  return accepted;
}

export function abortTask(taskId: string): Promise<boolean> {
  return runWorkforceAction(() => workforceAbortTask(taskId));
}

export function runDreamer(projectId: string): Promise<boolean> {
  return runWorkforceAction(() => workforceDreamer(projectId));
}

interface WorkforceEvent {
  type: string;
  sessionId?: string | null;
}

let syncStarted = false;
let syncing = false;
let syncTimer: ReturnType<typeof setTimeout> | undefined;

async function syncNow(): Promise<void> {
  if (syncing) return;
  syncing = true;
  try {
    await workforceTick();
    const next = await workforcePoll();
    if (!next) return;
    app.snapshot = next;
    app.chat = next.chat ?? [];
    app.workforceError = null;
  } catch (error) {
    app.workforceError = String(error);
  } finally {
    syncing = false;
  }
}

function scheduleSync(): void {
  if (syncTimer) clearTimeout(syncTimer);
  syncTimer = setTimeout(() => void syncNow(), 150);
}

export function startWorkforceSync(): void {
  if (syncStarted || !workforceAvailable()) return;
  syncStarted = true;
  void listen<WorkforceEvent>("workforce-event", scheduleSync);
  // SSE is volatile by contract; this slower pass recovers missed events.
  setInterval(() => void syncNow(), 15_000);
}

export async function decideProposal(id: string, status: Extract<ProposalStatus, "approved" | "rejected">): Promise<void> {
  if (workforceAvailable()) {
    await runWorkforceAction(() => workforceDecideProposal(id, status));
    return;
  }
  // Browser-only preview: the decision lives in this session and is never written.
  const proposal = app.snapshot?.proposals.find((candidate) => candidate.id === id);
  if (!proposal || proposal.status !== "pending") return;
  proposal.status = status;
  proposal.decidedAt = Date.now();
}

export function go(view: View): void {
  app.view = view;
  app.paletteOpen = false;
  app.sheet = null;
}

export function openSheet(kind: SheetKind, id: string): void {
  app.sheet = { kind, id };
}

export function closeSheet(): void {
  app.sheet = null;
}

export function openTask(id: string): void {
  app.view = "tasks";
  app.sheet = { kind: "task", id };
  app.paletteOpen = false;
}

export function navigateTo(kind: SheetKind, id: string): void {
  if (kind === "task") {
    openTask(id);
    return;
  }
  app.view =
    kind === "agent"
      ? "agents"
      : kind === "project" || kind === "project-new"
        ? "projects"
        : "memory";
  app.sheet = { kind, id };
  app.paletteOpen = false;
}

export async function openOrchestrator(): Promise<void> {
  if (isTauri()) {
    try {
      await invoke("open_orchestrator");
      return;
    } catch (error) {
      console.warn("orchestrator window unavailable", error);
    }
  }
  app.overlayOpen = true;
}

export async function openPets(): Promise<void> {
  if (!isTauri()) return;
  try {
    await invoke("open_pets");
    app.petsVisible = true;
  } catch (error) {
    console.warn("pet windows unavailable", error);
  }
}

export async function hidePets(): Promise<void> {
  if (!isTauri()) return;
  try {
    await invoke("hide_pets");
    app.petsVisible = false;
  } catch (error) {
    console.warn("pet windows unavailable", error);
  }
}

export function togglePets(): void {
  void (app.petsVisible ? hidePets() : openPets());
}

export async function closeOrchestrator(): Promise<void> {
  if (isTauri()) {
    try {
      await invoke("hide_orchestrator");
      return;
    } catch (error) {
      console.warn("orchestrator window unavailable", error);
    }
  }
  app.overlayOpen = false;
}
