export type RuntimeItemKind = "user" | "assistant" | "tool" | "system";

export interface RuntimeItem {
  id: string;
  kind: RuntimeItemKind;
  text: string;
  toolName?: string;
  toolStatus?: string;
  input?: string;
  output?: string;
}

interface MessageLike {
  id?: string;
  type?: string;
  text?: string;
  time?: { created?: number };
  content?: Array<{
    type?: string;
    text?: string;
    name?: string;
    state?: {
      status?: string;
      input?: unknown;
      content?: Array<{ type?: string; text?: string }>;
    };
  }>;
}

function pretty(value: unknown): string {
  if (value === undefined || value === null) return "";
  if (typeof value === "string") return value;
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function toolOutput(content: Array<{ type?: string; text?: string }> | undefined): string {
  if (!content) return "";
  return content
    .map((part) => (part.type === "text" ? part.text ?? "" : ""))
    .filter(Boolean)
    .join("\n");
}

export function normalizeRuntimeMessages(messages: unknown[]): RuntimeItem[] {
  const items: RuntimeItem[] = [];

  for (const raw of messages) {
    const message = raw as MessageLike;
    if (!message || typeof message !== "object") continue;
    const id = message.id ?? `message-${items.length}`;

    if (message.type === "user") {
      items.push({ id, kind: "user", text: message.text ?? "" });
      continue;
    }

    if (message.type === "assistant" && Array.isArray(message.content)) {
      let index = 0;
      for (const part of message.content) {
        const partId = `${id}-${index++}`;
        if (part.type === "text" && part.text) {
          items.push({ id: partId, kind: "assistant", text: part.text });
        }
        if (part.type === "tool") {
          items.push({
            id: partId,
            kind: "tool",
            text: part.name ?? "tool",
            toolName: part.name ?? "tool",
            toolStatus: part.state?.status ?? "unknown",
            input: pretty(part.state?.input),
            output: toolOutput(part.state?.content),
          });
        }
      }
    }
  }

  return items;
}

export function normalizePermissions(raw: unknown[]): Array<{
  id: string;
  sessionID: string;
  action: string;
  resources: string[];
  message?: string;
}> {
  return raw
    .map((entry) => entry as Record<string, unknown>)
    .filter((entry) => typeof entry.id === "string" && typeof entry.action === "string")
    .map((entry) => ({
      id: entry.id as string,
      sessionID: (entry.sessionID as string) ?? "",
      action: entry.action as string,
      resources: Array.isArray(entry.resources) ? (entry.resources as string[]) : [],
      message: typeof entry.message === "string" ? entry.message : undefined,
    }));
}

export function modelKey(providerID: string, modelID: string): string {
  return `${providerID}::${modelID}`;
}
