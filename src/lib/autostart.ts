import { isTauri } from "@tauri-apps/api/core";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";

const CONFIGURED_KEY = "agent-bots:autostart-configured";

export async function autostartEnabled(): Promise<boolean> {
  if (!isTauri()) return false;
  return isEnabled();
}

export async function setAutostart(enabled: boolean): Promise<boolean> {
  if (!isTauri()) return false;
  if (enabled) await enable();
  else await disable();
  localStorage.setItem(CONFIGURED_KEY, enabled ? "on" : "off");
  return isEnabled();
}

export async function ensureAutostart(): Promise<void> {
  if (!isTauri()) return;
  if (localStorage.getItem(CONFIGURED_KEY)) return;
  try {
    if (!(await isEnabled())) await enable();
    localStorage.setItem(CONFIGURED_KEY, "on");
  } catch (error) {
    console.warn("autostart setup unavailable", error);
  }
}
