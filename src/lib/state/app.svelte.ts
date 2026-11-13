import { invoke, isTauri } from "@tauri-apps/api/core";
import type { ChatMessage, ProposalStatus, Snapshot } from "../types.ts";
import { loadSnapshot } from "../data/provider.ts";
import { readPolicy, type PolicyBundle } from "../policy.ts";
import type { View } from "../nav.ts";

export type SheetKind = "agent" | "project" | "task" | "memory" | "proposal";

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
  sheet: null as SheetRef | null,
  chat: [] as ChatMessage[],
  pending: false,
  policy: null as PolicyBundle | null,
  policyProjectId: null as string | null,
  policyLoading: false,
  policyError: null as string | null,
});

let started = false;

export async function init(): Promise<void> {
  if (started) return;
  started = true;
  const snapshot = await loadSnapshot();
  app.snapshot = snapshot;
  app.chat = snapshot.chat;
  await loadPolicy("core");
  app.ready = true;
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

export function decideProposal(id: string, status: Extract<ProposalStatus, "approved" | "rejected">): void {
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
  app.view = kind === "agent" ? "agents" : kind === "project" ? "projects" : "memory";
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
