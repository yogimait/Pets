import type { Snapshot } from "../types.ts";
import { snapshot } from "./index.ts";

// Single seam between the interface and workforce data.
// Replace this body with a Tauri invoke or HTTP client when a backend exists.
export async function loadSnapshot(): Promise<Snapshot> {
  return structuredClone(snapshot);
}
