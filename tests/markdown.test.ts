import test from "node:test";
import assert from "node:assert/strict";
import { inlineSegments, parseMarkdown } from "../src/lib/markdown.ts";

test("renders a markdown table as a table block", () => {
  const blocks = parseMarkdown(
    "Denial report\n\n| # | Action | Result |\n|---|---|---|\n| 1 | Read `.env` | **Denied** |",
  );
  assert.deepEqual(blocks[0], { type: "paragraph", text: "Denial report" });
  assert.deepEqual(blocks[1], {
    type: "table",
    header: ["#", "Action", "Result"],
    rows: [["1", "Read `.env`", "**Denied**"]],
  });
});

test("keeps fenced code together and parses headings and lists", () => {
  const blocks = parseMarkdown("# Result\n\n- first\n- second\n\n```json\n{\"ok\":true}\n```");
  assert.deepEqual(blocks[0], { type: "heading", level: 1, text: "Result" });
  assert.deepEqual(blocks[1], { type: "list", ordered: false, items: ["first", "second"] });
  assert.deepEqual(blocks[2], { type: "code", language: "json", text: "{\"ok\":true}" });
});

test("splits inline code and strong text without rendering html", () => {
  assert.deepEqual(inlineSegments("Read `<b>.env</b>` is **Denied**"), [
    { kind: "text", text: "Read " },
    { kind: "code", text: "<b>.env</b>" },
    { kind: "text", text: " is " },
    { kind: "strong", text: "Denied" },
  ]);
});
