import test from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { assemblePrompt, parsePrompt } from "../src/lib/prompts.ts";
import { snapshot } from "../src/lib/data/index.ts";

const agentsDir = fileURLToPath(new URL("../prompts/agents", import.meta.url));
const files = readdirSync(agentsDir).filter((file) => file.endsWith(".md"));
const expected = [
  "auditing",
  "backend",
  "dreamer",
  "frontend",
  "marketing",
  "orchestrator",
  "research",
  "testing",
  "ui-ux",
];

const prompts = files.map((file) => {
  const raw = readFileSync(`${agentsDir}/${file}`, "utf8");
  return parsePrompt(raw, file.replace(/\.md$/, ""));
});

test("every expected role has exactly one prompt", () => {
  assert.deepEqual(prompts.map((prompt) => prompt.meta.id).sort(), expected.sort());
});

test("prompt frontmatter and body are complete", () => {
  for (const prompt of prompts) {
    assert.ok(prompt.meta.name.length > 0, `${prompt.meta.id} has no name`);
    assert.ok(["parent", "memory", "specialist"].includes(prompt.meta.kind), `${prompt.meta.id} has invalid kind`);
    assert.ok(prompt.body.length > 500, `${prompt.meta.id} body is too short`);
    assert.match(prompt.body, /## (Mission|Mission and scope)/, `${prompt.meta.id} has no mission section`);
  }
  assert.equal(prompts.find((prompt) => prompt.meta.id === "orchestrator")?.meta.kind, "parent");
  assert.equal(prompts.find((prompt) => prompt.meta.id === "dreamer")?.meta.kind, "memory");
});

test("prompts do not copy platform-specific tools or paths", () => {
  const forbidden = ["cloud_threads", "automations.notify_parent", "$orbit:follow-up", "/agent_notes/", "/user_notes/"];
  for (const prompt of prompts) {
    for (const token of forbidden) {
      assert.ok(!prompt.raw.includes(token), `${prompt.meta.id} contains platform-specific token ${token}`);
    }
  }
});

test("assembler preserves policy and role order", () => {
  const assembled = assemblePrompt({
    globalPolicy: "GLOBAL_POLICY",
    projectPolicy: "PROJECT_POLICY",
    rolePrompt: "ROLE_PROMPT",
    handoff: "HANDOFF_CONTRACT",
  });
  const order = ["GLOBAL_POLICY", "PROJECT_POLICY", "ROLE_PROMPT", "HANDOFF_CONTRACT"].map((marker) =>
    assembled.indexOf(marker),
  );
  assert.ok(order.every((index) => index >= 0));
  assert.deepEqual(order, [...order].sort((a, b) => a - b));
  assert.match(assembled, /# Policy precedence/);
});

test("specialist prompt gets the handoff contract and Dreamer gets the memory contract", () => {
  const frontend = prompts.find((prompt) => prompt.meta.id === "frontend");
  const dreamer = prompts.find((prompt) => prompt.meta.id === "dreamer");
  assert.ok(frontend && dreamer);

  const specialist = assemblePrompt({ globalPolicy: "", rolePrompt: frontend.body, handoff: "HANDOFF_CONTRACT" });
  assert.match(specialist, /HANDOFF_CONTRACT/);
  assert.doesNotMatch(specialist, /MEMORY_PROPOSAL_CONTRACT/);

  const memory = assemblePrompt({
    globalPolicy: "",
    rolePrompt: dreamer.body,
    memoryProposal: "MEMORY_PROPOSAL_CONTRACT",
  });
  assert.match(memory, /MEMORY_PROPOSAL_CONTRACT/);
  assert.doesNotMatch(memory, /HANDOFF_CONTRACT/);
});

test("orchestrator routes every specialist", () => {
  const orchestrator = prompts.find((prompt) => prompt.meta.id === "orchestrator");
  assert.ok(orchestrator);
  const routing = orchestrator.body.toLowerCase();
  for (const role of ["frontend", "backend", "research", "marketing", "ui/ux", "testing", "auditing"]) {
    assert.ok(routing.includes(role), `orchestrator routing is missing ${role}`);
  }
});

test("every agent points at a registered prompt", () => {
  const ids = new Set(prompts.map((prompt) => prompt.meta.id));
  for (const agent of snapshot.agents) {
    assert.ok(ids.has(agent.promptId), `agent ${agent.id} points at missing prompt ${agent.promptId}`);
  }
});
