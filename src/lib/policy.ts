import { invoke, isTauri } from "@tauri-apps/api/core";

export interface PolicyFile {
  path: string;
  exists: boolean;
  content: string;
  hash: string;
  bytes: number;
  error: string | null;
}

export interface PolicyBundle {
  global: PolicyFile;
  project: PolicyFile | null;
  loadedAt: number;
}

const unavailable: PolicyFile = {
  path: "%USERPROFILE%\\.config\\opencode\\AGENTS.md",
  exists: false,
  content: "",
  hash: "",
  bytes: 0,
  error: "Policy files are readable only in the desktop app.",
};

export async function readPolicy(projectRoot?: string | null): Promise<PolicyBundle> {
  if (isTauri()) {
    return invoke<PolicyBundle>("read_policy", { projectRoot: projectRoot ?? null });
  }
  return {
    global: unavailable,
    project: projectRoot
      ? {
          path: `${projectRoot}\\AGENTS.md`,
          exists: false,
          content: "",
          hash: "",
          bytes: 0,
          error: "Policy files are readable only in the desktop app.",
        }
      : null,
    loadedAt: Date.now(),
  };
}
