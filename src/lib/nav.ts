import type { Component } from "svelte";
import { Activity, Bot, Brain, FolderKanban, LayoutDashboard, ListChecks, Settings } from "@lucide/svelte";

export type View = "overview" | "agents" | "projects" | "tasks" | "activity" | "memory" | "settings";

export interface NavItem {
  id: View;
  label: string;
  hint: string;
  icon: Component;
}

export const NAV: NavItem[] = [
  { id: "overview", label: "Overview", hint: "Workforce state", icon: LayoutDashboard },
  { id: "agents", label: "Agents", hint: "Specialists", icon: Bot },
  { id: "projects", label: "Projects", hint: "Workspaces", icon: FolderKanban },
  { id: "tasks", label: "Tasks", hint: "Execution board", icon: ListChecks },
  { id: "activity", label: "Activity", hint: "Unified feed", icon: Activity },
  { id: "memory", label: "Memory", hint: "Dreamer ledger", icon: Brain },
  { id: "settings", label: "Settings", hint: "Providers and data", icon: Settings },
];

export const VIEW_TITLES: Record<View, string> = {
  overview: "Overview",
  agents: "Agents",
  projects: "Projects",
  tasks: "Tasks",
  activity: "Activity",
  memory: "Memory",
  settings: "Settings",
};
