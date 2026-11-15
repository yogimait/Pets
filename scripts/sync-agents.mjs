import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const sourceDir = join(root, "prompts", "agents");
const targetDir = join(root, ".opencode", "agents");
const globalDir =
  process.env.AGENT_BOTS_GLOBAL_AGENT_DIR ?? join(homedir(), ".config", "opencode", "agent");
const model = process.env.AGENT_BOTS_AGENT_MODEL ?? "opencode-go/deepseek-v4.1-flash";

const roles = [
  {
    id: "orchestrator-workforce",
    source: "orchestrator",
    name: "Workforce Orchestrator",
    description: "Plans work, delegates tasks, and reports progress for the Agent Bots desktop app.",
    permissions: "planning",
    suffix:
      "You are running inside the Agent Bots desktop app. Return only the answer or JSON the app requests. Do not use shell commands, the legacy taskboard, or subagents. The app owns tasks, sessions, memory, and approvals.",
  },
  {
    id: "dreamer",
    source: "dreamer",
    name: "Dreamer",
    description: "Maintains durable project memory and returns evidence-backed proposals.",
    permissions: "read-only",
  },
  {
    id: "pixel",
    source: "frontend",
    name: "Pixel",
    description: "Frontend implementation agent for interface work, components, and accessibility.",
    permissions: "workspace",
  },
  {
    id: "forge",
    source: "backend",
    name: "Forge",
    description: "Backend implementation agent for APIs, data models, migrations, and integrations.",
    permissions: "workspace",
  },
  {
    id: "scout",
    source: "research",
    name: "Scout",
    description: "Research agent for evidence, sources, comparisons, and short memos.",
    permissions: "workspace",
  },
  {
    id: "voice",
    source: "marketing",
    name: "Voice",
    description: "Marketing agent for positioning, launch copy, and claim review.",
    permissions: "workspace",
  },
  {
    id: "probe",
    source: "testing",
    name: "Probe",
    description: "Testing agent for verification, regression checks, and release gates.",
    permissions: "workspace",
  },
  {
    id: "iris",
    source: "ui-ux",
    name: "Iris",
    description: "UI/UX agent for flows, states, interaction specs, and accessibility review.",
    permissions: "workspace",
  },
  {
    id: "verity",
    source: "auditing",
    name: "Verity",
    description: "Read-only auditing agent for security, permissions, data integrity, and release risk.",
    permissions: "audit",
  },
];

const mcpAccess = [
  ["playwright_*", "*", "allow"],
  ["n8n_mcp_*", "*", "allow"],
  ["stitch_*", "*", "allow"],
  ["rea_*", "*", "allow"],
  ["jev_*", "*", "allow"],
  ["laya_*", "*", "allow"],
];

const destructiveShell = [
  ["shell", "rm -rf *", "deny"],
  ["shell", "*Remove-Item*", "deny"],
  ["shell", "Remove-Item *", "deny"],
  ["shell", "del *", "deny"],
  ["shell", "rmdir *", "deny"],
  ["shell", "git push *", "deny"],
  ["shell", "git reset --hard *", "deny"],
  ["shell", "git clean -f*", "deny"],
  ["shell", "format *", "deny"],
  ["shell", "shutdown *", "deny"],
  ["shell", "Stop-Computer *", "deny"],
  ["shell", "Restart-Computer *", "deny"],
];

const permissionSets = {
  planning: [
    ["*", "*", "allow"],
    ...mcpAccess,
    ["read", "*.env", "deny"],
    ["read", "*.env.*", "deny"],
    ["read", "*.env.example", "allow"],
    ["shell", "*", "deny"],
    ...destructiveShell,
    ["edit", "*", "deny"],
    ["subagent", "*", "deny"],
    ["question", "*", "deny"],
    ["external_directory", "*", "deny"],
  ],
  "read-only": [
    ["*", "*", "allow"],
    ...mcpAccess,
    ["read", "*.env", "deny"],
    ["read", "*.env.*", "deny"],
    ["read", "*.env.example", "allow"],
    ["shell", "*", "deny"],
    ...destructiveShell,
    ["edit", "*", "deny"],
    ["subagent", "*", "deny"],
    ["question", "*", "deny"],
    ["external_directory", "*", "deny"],
  ],
  audit: [
    ["*", "*", "allow"],
    ...mcpAccess,
    ["read", "*.env", "deny"],
    ["read", "*.env.*", "deny"],
    ["read", "*.env.example", "allow"],
    ["edit", "*", "deny"],
    ["subagent", "*", "deny"],
    ["question", "*", "deny"],
    ["external_directory", "*", "deny"],
    ...destructiveShell,
  ],
  workspace: [
    ["*", "*", "allow"],
    ...mcpAccess,
    ["read", "*.env", "deny"],
    ["read", "*.env.*", "deny"],
    ["read", "*.env.example", "allow"],
    ["subagent", "*", "deny"],
    ["question", "*", "deny"],
    ["external_directory", "*", "deny"],
    ...destructiveShell,
  ],
};

function parsePrompt(raw) {
  const text = raw.replace(/^\uFEFF/, "");
  if (!text.startsWith("---")) return text.trim();
  const firstBreak = text.indexOf("\n");
  const closing = text.indexOf("\n---", firstBreak + 1);
  if (firstBreak < 0 || closing < 0) return text.trim();
  const afterClosing = text.indexOf("\n", closing + 1);
  return afterClosing >= 0 ? text.slice(afterClosing + 1).trim() : "";
}

function renderPermissions(entries) {
  return entries
    .map(
      ([action, resource, effect]) =>
        `  - action: ${JSON.stringify(action)}\n    resource: ${JSON.stringify(resource)}\n    effect: ${effect}`,
    )
    .join("\n");
}

const marker = "<!-- Generated by Agent Bots. Edit prompts/agents instead. -->";

mkdirSync(targetDir, { recursive: true });
mkdirSync(globalDir, { recursive: true });

for (const role of roles) {
  const raw = readFileSync(join(sourceDir, `${role.source}.md`), "utf8");
  const body = parsePrompt(raw);
  const suffix = role.suffix ? `\n\n## App runtime\n\n${role.suffix}` : "";
  const content = `---
description: ${JSON.stringify(role.description)}
mode: primary
model: ${model}
steps: 24
permissions:
${renderPermissions(permissionSets[role.permissions])}
---

${marker}

${body}${suffix}
`;

  writeFileSync(join(targetDir, `${role.id}.md`), content, "utf8");
  console.log(`wrote .opencode/agents/${role.id}.md`);

  const globalPath = join(globalDir, `${role.id}.md`);
  if (existsSync(globalPath)) {
    const existing = readFileSync(globalPath, "utf8");
    if (!existing.includes(marker)) {
      console.warn(`skipped global ${role.id}.md: existing file was not generated by Agent Bots`);
      continue;
    }
  }
  writeFileSync(globalPath, content, "utf8");
  console.log(`wrote global agent ${role.id}.md`);
}

console.log(`synced ${roles.length} agents with model ${model}`);
console.log(`global agent directory: ${globalDir}`);
