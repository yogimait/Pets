import type { Snapshot } from "../types.ts";
import { agents } from "./agents.ts";
import { projects } from "./projects.ts";
import { providers } from "./providers.ts";

const keepProjects = new Set(["core", "lt"]);

const realProjects = projects.filter((project) => keepProjects.has(project.id));
const realAgents = agents.map((agent) => ({
  ...agent,
  status: "idle" as const,
  projectIds: agent.projectIds.filter((id) => keepProjects.has(id)),
  currentTaskId: null,
  successRate: 0,
  tasksCompleted: 0,
  lastActiveAt: 0,
}));

export const seedSnapshot: Snapshot = {
  agents: realAgents,
  projects: realProjects,
  tasks: [],
  activity: [],
  memory: [],
  proposals: [],
  dreamerRuns: [],
  providers,
  artifacts: [],
  chat: [],
};
