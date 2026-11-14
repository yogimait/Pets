import { invoke, isTauri } from "@tauri-apps/api/core";

export interface RuntimeInfo {
  version: string;
  pid: number;
  port: number;
  connected: boolean;
}

export interface RuntimeAgent {
  id: string;
  name: string;
  description?: string;
  mode?: string;
  hidden?: boolean;
  model?: { id: string; providerID: string; variant?: string };
}

export interface RuntimeModel {
  id: string;
  modelID: string;
  providerID: string;
  name: string;
  enabled: boolean;
  capabilities?: { tools?: boolean };
  limit?: { context?: number; output?: number };
}

export interface RuntimeSession {
  id: string;
  title?: string;
  agent?: string;
  model?: { id: string; providerID: string; variant?: string };
  location?: { directory: string };
  time?: { created: number; updated: number; idle?: number };
}

export interface RuntimePermission {
  id: string;
  sessionID: string;
  action: string;
  resources: string[];
  message?: string;
  metadata?: Record<string, unknown>;
}

export function runtimeAvailable(): boolean {
  return isTauri();
}

export async function runtimeInfo(project: string): Promise<RuntimeInfo> {
  return invoke<RuntimeInfo>("runtime_info", { project });
}

export async function runtimeAgents(project: string): Promise<RuntimeAgent[]> {
  const result = await invoke<{ data: RuntimeAgent[] }>("runtime_agents", { project });
  return result.data ?? [];
}

export async function runtimeModels(project: string): Promise<RuntimeModel[]> {
  const result = await invoke<{ data: RuntimeModel[] }>("runtime_models", { project });
  return result.data ?? [];
}

export async function runtimeMcp(project: string): Promise<{ name: string; status: { status: string } }[]> {
  const result = await invoke<{ data: { name: string; status: { status: string } }[] }>("runtime_mcp", { project });
  return result.data ?? [];
}

export async function runtimeCreateSession(input: {
  project: string;
  title: string;
  agent: string;
  providerId: string;
  modelId: string;
  permissions?: unknown[];
}): Promise<RuntimeSession> {
  const result = await invoke<{ data: RuntimeSession }>("runtime_create_session", input);
  return result.data;
}

export async function runtimePrompt(sessionId: string, text: string): Promise<void> {
  await invoke("runtime_prompt", { sessionId, text });
}

export async function runtimeAbort(sessionId: string): Promise<void> {
  await invoke("runtime_abort", { sessionId });
}

export async function runtimeMessages(sessionId: string): Promise<unknown[]> {
  const result = await invoke<{ data: unknown[] }>("runtime_messages", { sessionId });
  return result.data ?? [];
}

export async function runtimePermissions(sessionId: string): Promise<RuntimePermission[]> {
  const result = await invoke<{ data: RuntimePermission[] }>("runtime_permissions", { sessionId });
  return result.data ?? [];
}

export async function runtimeReplyPermission(
  sessionId: string,
  requestId: string,
  decision: "once" | "always" | "reject",
): Promise<void> {
  await invoke("runtime_reply_permission", { sessionId, requestId, decision });
}

export async function runtimeStop(): Promise<void> {
  await invoke("runtime_stop");
}
