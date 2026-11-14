import test from "node:test";
import assert from "node:assert/strict";
import { modelKey, normalizePermissions, normalizeRuntimeMessages } from "../src/lib/runtime-format.ts";

test("runtime messages normalize text, tools, and user input", () => {
  const items = normalizeRuntimeMessages([
    {
      id: "m1",
      type: "user",
      text: "Build the component",
      time: { created: 1 },
    },
    {
      id: "m2",
      type: "assistant",
      content: [
        { type: "text", text: "I will inspect the code." },
        {
          type: "tool",
          name: "read",
          state: { status: "completed", input: { path: "src/App.svelte" }, content: [{ type: "text", text: "file body" }] },
        },
        { type: "reasoning", text: "hidden" },
      ],
      time: { created: 2 },
    },
  ]);

  assert.equal(items.length, 3);
  assert.deepEqual(
    items.map((item) => item.kind),
    ["user", "assistant", "tool"],
  );
  assert.equal(items[0].text, "Build the component");
  assert.equal(items[1].text, "I will inspect the code.");
  assert.equal(items[2].toolName, "read");
  assert.equal(items[2].toolStatus, "completed");
  assert.match(items[2].input ?? "", /src\/App\.svelte/);
  assert.equal(items[2].output, "file body");
});

test("runtime messages ignore idle and malformed entries", () => {
  const items = normalizeRuntimeMessages([
    null,
    { id: "idle", type: "idle", outcome: "succeeded" },
    { id: "empty", type: "assistant", content: [] },
  ]);
  assert.equal(items.length, 0);
});

test("permissions normalize required fields", () => {
  const permissions = normalizePermissions([
    { id: "p1", sessionID: "s1", action: "shell", resources: ["git status"] },
    { id: "p2", sessionID: "s1", action: "edit", resources: ["src/App.svelte"], message: "Write file" },
    { action: "edit", resources: [] },
  ]);
  assert.equal(permissions.length, 2);
  assert.equal(permissions[0].action, "shell");
  assert.deepEqual(permissions[0].resources, ["git status"]);
  assert.equal(permissions[1].message, "Write file");
});

test("model key avoids slash ambiguity", () => {
  assert.equal(modelKey("openrouter", "z-ai/glm-5.3"), "openrouter::z-ai/glm-5.3");
});
