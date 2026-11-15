export interface InlineSegment {
  kind: "text" | "code" | "strong";
  text: string;
}

export type MarkdownBlock =
  | { type: "paragraph"; text: string }
  | { type: "heading"; level: number; text: string }
  | { type: "code"; language: string; text: string }
  | { type: "list"; ordered: boolean; items: string[] }
  | { type: "table"; header: string[]; rows: string[][] }
  | { type: "quote"; text: string }
  | { type: "rule" };

const fence = /^\s*```(.*)$/;
const heading = /^\s*(#{1,6})\s+(.*)$/;
const bullet = /^\s*[-*+]\s+(.*)$/;
const ordered = /^\s*\d+[.)]\s+(.*)$/;
const quote = /^\s*>\s?(.*)$/;
const rule = /^\s*(?:-{3,}|\*{3,}|_{3,})\s*$/;
const separatorCell = /^:?-{3,}:?$/;

function splitRow(line: string): string[] {
  return line
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((cell) => cell.trim());
}

function isTableSeparator(line: string): boolean {
  if (!line.trim().startsWith("|")) return false;
  const cells = splitRow(line);
  return cells.length > 0 && cells.every((cell) => separatorCell.test(cell));
}

function isTableStart(lines: string[], index: number): boolean {
  return lines[index]?.trim().startsWith("|") === true && isTableSeparator(lines[index + 1] ?? "");
}

function isBlockStart(lines: string[], index: number): boolean {
  const line = lines[index] ?? "";
  const trimmed = line.trim();
  if (!trimmed) return true;
  if (fence.test(trimmed) || heading.test(trimmed) || quote.test(trimmed) || rule.test(trimmed)) return true;
  if (bullet.test(trimmed) || ordered.test(trimmed)) return true;
  return isTableStart(lines, index);
}

export function inlineSegments(value: string): InlineSegment[] {
  const segments: InlineSegment[] = [];
  const pattern = /(`[^`]+`|\*\*[^*]+\*\*|__[^_]+__)/g;
  let cursor = 0;
  for (const match of value.matchAll(pattern)) {
    const index = match.index ?? 0;
    if (index > cursor) segments.push({ kind: "text", text: value.slice(cursor, index) });
    const token = match[0];
    if (token.startsWith("`")) {
      segments.push({ kind: "code", text: token.slice(1, -1) });
    } else {
      segments.push({ kind: "strong", text: token.slice(2, -2) });
    }
    cursor = index + token.length;
  }
  if (cursor < value.length) segments.push({ kind: "text", text: value.slice(cursor) });
  return segments.length > 0 ? segments : [{ kind: "text", text: value }];
}

export function parseMarkdown(input: string): MarkdownBlock[] {
  const lines = input.replace(/\r\n?/g, "\n").split("\n");
  const blocks: MarkdownBlock[] = [];
  let index = 0;

  while (index < lines.length) {
    const line = lines[index] ?? "";
    const trimmed = line.trim();
    if (!trimmed) {
      index += 1;
      continue;
    }

    const fenceMatch = trimmed.match(fence);
    if (fenceMatch) {
      const language = fenceMatch[1].trim();
      const content: string[] = [];
      index += 1;
      while (index < lines.length && !(lines[index] ?? "").trim().startsWith("```")) {
        content.push(lines[index] ?? "");
        index += 1;
      }
      if (index < lines.length) index += 1;
      blocks.push({ type: "code", language, text: content.join("\n") });
      continue;
    }

    const headingMatch = trimmed.match(heading);
    if (headingMatch) {
      blocks.push({ type: "heading", level: headingMatch[1].length, text: headingMatch[2].trim() });
      index += 1;
      continue;
    }

    if (rule.test(trimmed)) {
      blocks.push({ type: "rule" });
      index += 1;
      continue;
    }

    if (isTableStart(lines, index)) {
      const header = splitRow(line);
      index += 2;
      const rows: string[][] = [];
      while (index < lines.length && (lines[index] ?? "").trim().startsWith("|")) {
        rows.push(splitRow(lines[index] ?? ""));
        index += 1;
      }
      blocks.push({ type: "table", header, rows });
      continue;
    }

    const quoteMatch = trimmed.match(quote);
    if (quoteMatch) {
      const content: string[] = [quoteMatch[1]];
      index += 1;
      while (index < lines.length) {
        const next = (lines[index] ?? "").trim().match(quote);
        if (!next) break;
        content.push(next[1]);
        index += 1;
      }
      blocks.push({ type: "quote", text: content.join("\n") });
      continue;
    }

    const orderedMatch = trimmed.match(ordered);
    const bulletMatch = trimmed.match(bullet);
    if (orderedMatch || bulletMatch) {
      const isOrdered = Boolean(orderedMatch);
      const items: string[] = [];
      while (index < lines.length) {
        const candidate = (lines[index] ?? "").trim();
        const match = isOrdered ? candidate.match(ordered) : candidate.match(bullet);
        if (!match) break;
        items.push(match[1]);
        index += 1;
      }
      blocks.push({ type: "list", ordered: isOrdered, items });
      continue;
    }

    const paragraph: string[] = [line];
    index += 1;
    while (index < lines.length && !isBlockStart(lines, index)) {
      paragraph.push(lines[index] ?? "");
      index += 1;
    }
    blocks.push({ type: "paragraph", text: paragraph.join("\n").trimEnd() });
  }

  return blocks;
}
