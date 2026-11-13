import handoffRaw from "../../prompts/shared/handoff.md?raw";
import memoryProposalRaw from "../../prompts/shared/memory-proposal.md?raw";
import { parsePrompt, type ParsedPrompt } from "./prompts.ts";

const modules = import.meta.glob("../../prompts/agents/*.md", {
  eager: true,
  query: "?raw",
  import: "default",
}) as Record<string, string>;

export const agentPrompts: ParsedPrompt[] = Object.entries(modules)
  .map(([path, raw]) => {
    const fallback = path.split("/").pop()?.replace(/\.md$/, "") ?? "unknown";
    return parsePrompt(raw, fallback);
  })
  .sort((a, b) => a.meta.id.localeCompare(b.meta.id));

export const handoffContract = handoffRaw;
export const memoryProposalContract = memoryProposalRaw;

export function findAgentPrompt(id: string): ParsedPrompt | undefined {
  return agentPrompts.find((prompt) => prompt.meta.id === id);
}
