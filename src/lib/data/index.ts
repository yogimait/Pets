import type { Snapshot } from "../types.ts";
import { agents } from "./agents.ts";
import { projects } from "./projects.ts";
import { tasks } from "./tasks.ts";
import { activity } from "./activity.ts";
import { memory } from "./memory.ts";
import { proposals } from "./proposals.ts";
import { dreamerRuns } from "./dreamer-runs.ts";
import { providers } from "./providers.ts";
import { artifacts } from "./artifacts.ts";
import { chat } from "./chat.ts";

export const snapshot: Snapshot = {
  agents,
  projects,
  tasks,
  activity,
  memory,
  proposals,
  dreamerRuns,
  providers,
  artifacts,
  chat,
};
