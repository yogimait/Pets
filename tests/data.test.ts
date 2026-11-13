import test from "node:test";
import assert from "node:assert/strict";
import { snapshot } from "../src/lib/data/index.ts";
import {
  activityFiltered,
  buildMetrics,
  memoryFiltered,
  modelLabel,
  pendingProposals,
  projectStats,
  taskStepProgress,
  tasksByProject,
} from "../src/lib/selectors.ts";

const agentIds = new Set(snapshot.agents.map((agent) => agent.id));
const projectIds = new Set(snapshot.projects.map((project) => project.id));
const taskIds = new Set(snapshot.tasks.map((task) => task.id));
const memoryIds = new Set(snapshot.memory.map((entry) => entry.id));

test("every task points at a real owner and project", () => {
  for (const task of snapshot.tasks) {
    assert.ok(agentIds.has(task.ownerId), `task ${task.id} has unknown owner ${task.ownerId}`);
    assert.ok(projectIds.has(task.projectId), `task ${task.id} has unknown project ${task.projectId}`);
  }
});

test("activity and memory references resolve", () => {
  for (const event of snapshot.activity) {
    assert.ok(agentIds.has(event.actorId), `event ${event.id} has unknown actor`);
    if (event.projectId) assert.ok(projectIds.has(event.projectId), `event ${event.id} has unknown project`);
    if (event.taskId) assert.ok(taskIds.has(event.taskId), `event ${event.id} has unknown task`);
  }
  for (const entry of snapshot.memory) {
    assert.ok(projectIds.has(entry.projectId), `memory ${entry.id} has unknown project`);
    if (entry.linkedTaskId) assert.ok(taskIds.has(entry.linkedTaskId), `memory ${entry.id} has unknown task`);
    assert.ok(entry.confidence >= 0 && entry.confidence <= 1, `memory ${entry.id} confidence out of range`);
  }
});

test("identifiers are unique", () => {
  for (const [label, values] of [
    ["agents", snapshot.agents.map((agent) => agent.id)],
    ["projects", snapshot.projects.map((project) => project.id)],
    ["tasks", snapshot.tasks.map((task) => task.id)],
    ["activity", snapshot.activity.map((event) => event.id)],
    ["memory", snapshot.memory.map((entry) => entry.id)],
    ["proposals", snapshot.proposals.map((proposal) => proposal.id)],
    ["dreamer runs", snapshot.dreamerRuns.map((run) => run.id)],
  ] as const) {
    assert.equal(new Set(values).size, values.length, `${label} contain duplicate ids`);
  }
});

test("an agent's current task is running and owned by that agent", () => {
  for (const agent of snapshot.agents) {
    if (!agent.currentTaskId) continue;
    const task = snapshot.tasks.find((candidate) => candidate.id === agent.currentTaskId);
    assert.ok(task, `agent ${agent.id} points at a missing task`);
    assert.equal(task.status, "running", `agent ${agent.id} current task is not running`);
    assert.equal(task.ownerId, agent.id, `agent ${agent.id} current task has another owner`);
  }
});

test("every agent model resolves to a configured provider", () => {
  for (const agent of snapshot.agents) {
    const provider = snapshot.providers.find((candidate) => candidate.id === agent.providerId);
    assert.ok(provider, `agent ${agent.id} has unknown provider`);
    assert.ok(provider.configured, `agent ${agent.id} uses an unconfigured provider`);
    const label = modelLabel(snapshot, agent);
    assert.ok(label.model.length > 0);
  }
});

test("metrics agree with task statuses", () => {
  const metrics = buildMetrics(snapshot);
  assert.equal(metrics.running, snapshot.tasks.filter((task) => task.status === "running").length);
  assert.equal(metrics.blocked, snapshot.tasks.filter((task) => task.status === "blocked").length);
  assert.equal(metrics.memoryEntries, snapshot.memory.length);
  assert.equal(metrics.activeAgents, snapshot.agents.filter((agent) => agent.status === "active").length);
});

test("project progress stays within bounds", () => {
  for (const project of snapshot.projects) {
    const stats = projectStats(snapshot, project.id);
    assert.equal(stats.total, tasksByProject(snapshot, project.id).length);
    assert.ok(stats.progress >= 0 && stats.progress <= 1);
  }
});

test("task step progress stays within bounds", () => {
  for (const task of snapshot.tasks) {
    const progress = taskStepProgress(task);
    assert.ok(progress >= 0 && progress <= 1);
  }
});

test("filters return strict subsets", () => {
  assert.ok(activityFiltered(snapshot, { kind: "memory" }).every((event) => event.kind === "memory"));
  assert.ok(activityFiltered(snapshot, { projectId: "vault" }).every((event) => event.projectId === "vault"));
  assert.ok(memoryFiltered(snapshot, { projectId: "core" }).every((entry) => entry.projectId === "core"));
  assert.ok(memoryFiltered(snapshot, { query: "tauri" }).length > 0);
  assert.equal(memoryFiltered(snapshot, { query: "zzz-no-match" }).length, 0);
});

test("orchestrator thread references real entities", () => {
  for (const message of snapshot.chat) {
    for (const ref of message.refs) {
      if (ref.kind === "task") assert.ok(taskIds.has(ref.id), `chat ref has unknown task ${ref.id}`);
      if (ref.kind === "agent") assert.ok(agentIds.has(ref.id), `chat ref has unknown agent ${ref.id}`);
      if (ref.kind === "project") assert.ok(projectIds.has(ref.id), `chat ref has unknown project ${ref.id}`);
    }
  }
});

test("memory proposals and dreamer runs resolve", () => {
  for (const proposal of snapshot.proposals) {
    assert.ok(projectIds.has(proposal.projectId), `proposal ${proposal.id} has unknown project`);
    if (proposal.agentId) assert.ok(agentIds.has(proposal.agentId), `proposal ${proposal.id} has unknown agent`);
    if (proposal.targetEntryId) assert.ok(memoryIds.has(proposal.targetEntryId), `proposal ${proposal.id} has unknown target`);
    assert.ok(proposal.evidence.length > 0, `proposal ${proposal.id} has no evidence`);
    assert.ok(proposal.confidence >= 0 && proposal.confidence <= 1, `proposal ${proposal.id} confidence out of range`);
    if (proposal.status !== "pending") assert.ok(proposal.decidedAt, `proposal ${proposal.id} is decided without a timestamp`);
  }

  for (const run of snapshot.dreamerRuns) {
    assert.ok(projectIds.has(run.projectId), `run ${run.id} has unknown project`);
    if (run.finishedAt) assert.ok(run.finishedAt >= run.startedAt, `run ${run.id} finished before it started`);
    for (const proposalId of run.proposalIds) {
      assert.ok(snapshot.proposals.some((proposal) => proposal.id === proposalId), `run ${run.id} has unknown proposal`);
    }
  }

  assert.ok(pendingProposals(snapshot).length > 0);
});
