import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const agentsDir = join(root, ".opencode", "agents");
const globalDir =
  process.env.AGENT_BOTS_GLOBAL_AGENT_DIR ?? join(homedir(), ".config", "opencode", "agent");
const devVaultDir = process.env.DEVVAULT_PATH ?? join(homedir(), "Obsidian", "DevVault");

const expected = [
  "orchestrator-workforce",
  "dreamer",
  "pixel",
  "forge",
  "scout",
  "voice",
  "probe",
  "iris",
  "verity",
];

const failures = [];

for (const id of expected) {
  const path = join(agentsDir, `${id}.md`);
  if (!existsSync(path)) {
    failures.push(`${id}: missing ${path}`);
    continue;
  }

  const raw = readFileSync(path, "utf8");
  if (!raw.startsWith("---\n")) failures.push(`${id}: missing frontmatter`);
  if (!/^mode: primary$/m.test(raw)) failures.push(`${id}: mode must be primary`);
  if (!/^model: \S+\/\S+$/m.test(raw)) failures.push(`${id}: model must be provider/model`);
  if (!/^description: .+$/m.test(raw)) failures.push(`${id}: missing description`);
  if (!/^permissions:$/m.test(raw)) failures.push(`${id}: missing permissions`);

  const body = raw.split(/\n---\n/)[1] ?? "";
  if (body.trim().length < 500) failures.push(`${id}: prompt body is too short`);
  if (/taskboard\.py|cloud_threads|automations\.notify_parent/.test(raw)) {
    failures.push(`${id}: contains a legacy or platform-specific task command`);
  }
  if (/sk-[A-Za-z0-9]{8,}|Bearer\s+[A-Za-z0-9._-]{8,}|apiKey\s*[:=]/.test(raw)) {
    failures.push(`${id}: contains a secret-like literal`);
  }
  for (const tool of ["playwright", "n8n_mcp", "stitch", "rea", "jev", "laya"]) {
    const rule = new RegExp(`action: "${tool}_\\*"[\\s\\S]*?effect: allow`);
    if (!rule.test(raw)) failures.push(`${id}: ${tool}_* must be allow`);
  }
  if (!/resource: "\*Remove-Item\*"\s*\n\s*effect: deny/.test(raw)) {
    failures.push(`${id}: destructive Remove-Item shell command must be denied`);
  }
  if (!raw.includes(JSON.stringify(devVaultDir))) {
    failures.push(`${id}: DevVault external directory must be allowed`);
  }

  const globalPath = join(globalDir, `${id}.md`);
  if (!existsSync(globalPath)) {
    failures.push(`${id}: missing global agent ${globalPath}`);
  }
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(`checked ${expected.length} generated OpenCode agents`);
