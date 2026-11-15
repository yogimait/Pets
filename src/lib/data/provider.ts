import { isTauri } from "@tauri-apps/api/core";
import type { Snapshot } from "../types.ts";
import { seedSnapshot } from "./seed.ts";
import { workforceSeed, workforceSnapshot } from "../workforce.ts";

// Single seam between the interface and workforce data.
// Both desktop and browser start from the real empty workspace seed.
export async function loadSnapshot(): Promise<Snapshot> {
  if (isTauri()) {
    const stored = await workforceSnapshot();
    if (stored) return stored;
    return workforceSeed(structuredClone(seedSnapshot));
  }
  return structuredClone(seedSnapshot);
}
