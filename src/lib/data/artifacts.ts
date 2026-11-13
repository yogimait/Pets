import type { Artifact } from "../types.ts";

export const artifacts: Artifact[] = [
  { id: "art-01", projectId: "core", name: "tauri.conf.json", kind: "file", path: "src-tauri/tauri.conf.json" },
  { id: "art-02", projectId: "core", name: "Design system", kind: "doc", path: "docs/Design-System.md" },
  { id: "art-03", projectId: "core", name: "Token set", kind: "file", path: "src/app.css" },
  { id: "art-04", projectId: "vault", name: "0007_entitlements.sql", kind: "file", path: "migrations/0007_entitlements.sql" },
  { id: "art-05", projectId: "vault", name: "Billing API contract", kind: "doc", path: "docs/Billing-API.md" },
  { id: "art-06", projectId: "vault", name: "retry-queue.ts", kind: "file", path: "src/webhooks/retry-queue.ts" },
  { id: "art-07", projectId: "field-notes", name: "Inference cost memo", kind: "doc", path: "reports/inference-cost.md" },
  { id: "art-08", projectId: "launch", name: "Hero copy v3", kind: "doc", path: "copy/hero-v3.md" },
  { id: "art-09", projectId: "launch", name: "Pricing preview", kind: "url", path: "https://atlas.example.com/pricing" },
];
