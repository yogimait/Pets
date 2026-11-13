import type { Provider } from "../types.ts";

export const providers: Provider[] = [
  {
    id: "anthropic",
    name: "Anthropic",
    kind: "cloud",
    endpoint: "api.anthropic.com",
    configured: true,
    models: [
      { id: "claude-sonnet-4.5", name: "Claude Sonnet 4.5", context: 200_000, note: "Default for implementation work" },
      { id: "claude-haiku-4.5", name: "Claude Haiku 4.5", context: 200_000, note: "Cheap verification passes" },
    ],
  },
  {
    id: "openai",
    name: "OpenAI",
    kind: "cloud",
    endpoint: "api.openai.com",
    configured: true,
    models: [
      { id: "gpt-5.2", name: "GPT-5.2", context: 400_000, note: "Interface and research tasks" },
      { id: "gpt-5-mini", name: "GPT-5 mini", context: 400_000, note: "Copy and summarization" },
    ],
  },
  {
    id: "local",
    name: "Local runtime",
    kind: "local",
    endpoint: "http://127.0.0.1:11434",
    configured: true,
    models: [
      { id: "qwen3-30b-instruct", name: "Qwen3 30B Instruct", context: 32_000, note: "Memory consolidation, 4-bit" },
      { id: "llama-4-scout", name: "Llama 4 Scout", context: 128_000, note: "Offline fallback" },
    ],
  },
  {
    id: "openrouter",
    name: "OpenRouter",
    kind: "cloud",
    endpoint: "openrouter.ai/api",
    configured: false,
    models: [
      { id: "deepseek-v4", name: "DeepSeek V4", context: 164_000, note: "Not enabled" },
    ],
  },
];
