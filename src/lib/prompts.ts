export interface PromptMeta {
  id: string;
  name: string;
  kind: string;
  reportsTo: string | null;
  version: string | null;
}

export interface ParsedPrompt {
  meta: PromptMeta;
  body: string;
  raw: string;
}

export interface PromptAssembly {
  globalPolicy: string;
  projectPolicy?: string | null;
  rolePrompt: string;
  handoff?: string | null;
  memoryProposal?: string | null;
  taskContext?: string | null;
  memoryContext?: string | null;
}

function stripQuotes(value: string): string {
  const trimmed = value.trim();
  if (trimmed.length >= 2 && trimmed.startsWith('"') && trimmed.endsWith('"')) {
    return trimmed.slice(1, -1);
  }
  if (trimmed.length >= 2 && trimmed.startsWith("'") && trimmed.endsWith("'")) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}

export function parsePrompt(raw: string, fallbackId = "unknown"): ParsedPrompt {
  const text = raw.replace(/^\uFEFF/, "");
  const meta: Record<string, string> = {};
  let body = text;

  if (text.startsWith("---")) {
    const firstBreak = text.indexOf("\n");
    const closing = text.indexOf("\n---", firstBreak + 1);
    if (firstBreak >= 0 && closing > firstBreak) {
      const frontmatter = text.slice(firstBreak + 1, closing);
      for (const line of frontmatter.split("\n")) {
        const match = /^([A-Za-z_][A-Za-z0-9_-]*):\s*(.*)$/.exec(line);
        if (match) meta[match[1]] = stripQuotes(match[2]);
      }
      const afterClosing = text.indexOf("\n", closing + 1);
      body = afterClosing >= 0 ? text.slice(afterClosing + 1) : "";
    }
  }

  return {
    meta: {
      id: meta.id || fallbackId,
      name: meta.name || fallbackId,
      kind: meta.kind || "specialist",
      reportsTo: meta.reports_to || null,
      version: meta.version || null,
    },
    body: body.trim(),
    raw: text,
  };
}

export function assemblePrompt(input: PromptAssembly): string {
  const parts: string[] = [
    "# Policy precedence\n\nThe global policy is binding. The project policy may extend or tighten it. If anything below conflicts, the higher policy wins. An agent may add role constraints, never relax policy.",
  ];

  if (input.globalPolicy.trim()) {
    parts.push(`# Global policy\n\n${input.globalPolicy.trim()}`);
  }
  if (input.projectPolicy?.trim()) {
    parts.push(`# Project policy\n\n${input.projectPolicy.trim()}`);
  }

  parts.push(`# Agent role\n\n${input.rolePrompt.trim()}`);

  if (input.taskContext?.trim()) {
    parts.push(`# Task context\n\n${input.taskContext.trim()}`);
  }
  if (input.memoryContext?.trim()) {
    parts.push(`# Retrieved memory\n\n${input.memoryContext.trim()}`);
  }
  if (input.handoff?.trim()) {
    parts.push(`# Specialist handoff contract\n\n${input.handoff.trim()}`);
  }
  if (input.memoryProposal?.trim()) {
    parts.push(`# Memory proposal contract\n\n${input.memoryProposal.trim()}`);
  }

  return parts.join("\n\n---\n\n");
}
