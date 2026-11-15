import { isTauri } from "@tauri-apps/api/core";
import type { Snapshot } from "../types.ts";
import { snapshot } from "./index.ts";
import { workforceSeed, workforceSnapshot } from "../workforce.ts";

// Single seam between the interface and workforce data.
// In the desktop app the Rust core owns the snapshot; browser-only development keeps fixtures.
export async function loadSnapshot(): Promise<Snapshot> {
  if (isTauri()) {
    const stored = await workforceSnapshot();
    if (stored) return stored;
    return workforceSeed(structuredClone(snapshot));
  }
  return structuredClone(snapshot);
}
