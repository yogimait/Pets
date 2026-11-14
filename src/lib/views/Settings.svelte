<script lang="ts">
  import Panel from "../components/Panel.svelte";
  import StatusBadge from "../components/StatusBadge.svelte";
  import Avatar from "../components/Avatar.svelte";
  import RuntimePanel from "../components/RuntimePanel.svelte";
  import { modelLabel } from "../selectors.ts";
  import { app, loadPolicy, reloadPolicy } from "../state/app.svelte.ts";
  import { fullTimestamp } from "../format.ts";
  import type { Snapshot } from "../types.ts";

  let { snapshot }: { snapshot: Snapshot } = $props();

  function changePolicyProject(event: Event) {
    const select = event.currentTarget as HTMLSelectElement;
    loadPolicy(select.value);
  }

  const tokens = [
    { name: "Canvas", value: "#0a0b0d" },
    { name: "Surface", value: "#111316" },
    { name: "Raised", value: "#16181c" },
    { name: "Line", value: "#23262b" },
    { name: "Ink", value: "#e6e8eb" },
    { name: "Muted", value: "#71777d" },
    { name: "Accent", value: "#8ca6c0" },
  ];

  const configuredCount = $derived(snapshot.providers.filter((provider) => provider.configured).length);
</script>

<div class="space-y-5">
  <div>
    <h1 class="text-[18px] font-medium tracking-[-0.01em] text-ink">Settings</h1>
    <p class="mt-1 text-[11.5px] text-ink-3">Workspace data is still mock. The OpenCode runtime panel is live.</p>
  </div>

  <Panel title="Data source" subtitle="Interface only">
    <div class="flex items-start gap-3 rounded-control border border-warn/25 bg-warn/5 p-3">
      <span class="mt-1.5 size-1.5 shrink-0 rounded-full bg-warn" aria-hidden="true"></span>
      <div>
        <p class="text-[11.5px] text-ink">Local mock snapshot</p>
        <p class="mt-1 text-[10.5px] leading-relaxed text-ink-3">
          Every agent, task, activity event, and memory entry on the mock screens comes from one typed fixture set.
          The OpenCode runtime panel below is separate and can start real model sessions.
        </p>
      </div>
    </div>
    <ul class="mt-3 space-y-1.5 text-[10.5px] leading-relaxed text-ink-3">
      <li>
        <span class="text-ink-4">Provider seam</span> ·
        <span class="font-mono text-ink-2">src/lib/data/provider.ts</span> is the only file that changes for a real backend.
      </li>
      <li><span class="text-ink-4">Orchestrator</span> · the overlay is scripted. The OpenCode runtime panel starts real sessions separately.</li>
      <li><span class="text-ink-4">Persistence</span> · app state is not persisted. OpenCode sessions live outside the app.</li>
      <li><span class="text-ink-4">Agents</span> · cards and task data remain mock. A real session can be started below.</li>
    </ul>
  </Panel>

  <Panel title="Global policy" subtitle="Loaded verbatim from the OpenCode config">
    {#snippet action()}
      <button
        onclick={reloadPolicy}
        disabled={app.policyLoading}
        class="rounded border border-line-soft px-1.5 py-0.5 text-[10px] text-ink-3 transition-colors hover:text-ink disabled:opacity-40"
      >
        {app.policyLoading ? "Loading" : "Reload"}
      </button>
    {/snippet}
    <div class="space-y-3">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0">
          <p class="break-all font-mono text-[10px] text-ink-2">{app.policy?.global.path ?? "Loading policy…"}</p>
          <p class="mt-1 font-mono text-[9.5px] text-ink-4">
            {app.policy
              ? `${app.policy.global.bytes} bytes · ${app.policy.global.hash || "no hash"} · loaded ${fullTimestamp(app.policy.loadedAt)}`
              : "No policy loaded."}
          </p>
        </div>
        <StatusBadge
          status={app.policy?.global.exists ? "active" : "paused"}
          label={app.policy?.global.exists ? "Loaded" : "Missing"}
        />
      </div>

      <div class="flex flex-wrap items-center gap-2 border-t border-line-soft pt-3">
        <select
          value={app.policyProjectId ?? "core"}
          onchange={changePolicyProject}
          aria-label="Policy project"
          class="h-7 rounded-control border border-line-soft bg-surface px-1.5 text-[11px] text-ink-2 focus:border-accent/40 focus:outline-none"
        >
          {#each snapshot.projects as project (project.id)}
            <option value={project.id}>{project.key} · {project.name}</option>
          {/each}
        </select>
        <span class="min-w-0 flex-1 truncate font-mono text-[9.5px] text-ink-4">
          {app.policy?.project?.path ?? "No project AGENTS.md loaded"}
        </span>
        <StatusBadge
          status={app.policy?.project?.exists ? "active" : "paused"}
          label={app.policy?.project?.exists ? "Project policy" : "No project policy"}
        />
      </div>

      {#if app.policyError}
        <p class="rounded-control border border-bad/30 bg-bad/5 px-2.5 py-2 text-[10px] text-bad">{app.policyError}</p>
      {:else if app.policy?.global.error}
        <p class="rounded-control border border-warn/30 bg-warn/5 px-2.5 py-2 text-[10px] text-warn">
          {app.policy.global.error}
        </p>
      {/if}

      <p class="text-[10px] leading-relaxed text-ink-4">
        The harness loads this file before every agent prompt. The project file may add or tighten rules. Missing policy blocks a real run; the interface preview shows the state instead.
      </p>
    </div>
  </Panel>

  <RuntimePanel />

  <Panel title="Providers and models" subtitle={`${configuredCount} of ${snapshot.providers.length} configured`} padded={false}>
    <div class="overflow-x-auto">
      <table class="w-full border-collapse text-left">
        <thead>
          <tr class="border-b border-line-soft text-[9.5px] uppercase tracking-[0.1em] text-ink-4">
            <th class="px-3.5 py-2 font-medium">Provider</th>
            <th class="px-2 py-2 font-medium">Kind</th>
            <th class="px-2 py-2 font-medium">Endpoint</th>
            <th class="px-2 py-2 font-medium">Status</th>
            <th class="px-3.5 py-2 font-medium">Models</th>
          </tr>
        </thead>
        <tbody>
          {#each snapshot.providers as provider (provider.id)}
            <tr class="border-b border-line-soft last:border-0">
              <td class="whitespace-nowrap px-3.5 py-2.5 text-[11.5px] text-ink">{provider.name}</td>
              <td class="px-2 py-2.5 font-mono text-[10px] text-ink-3">{provider.kind}</td>
              <td class="px-2 py-2.5 font-mono text-[10px] text-ink-3">{provider.endpoint}</td>
              <td class="whitespace-nowrap px-2 py-2.5">
                <StatusBadge
                  status={provider.configured ? "active" : "paused"}
                  label={provider.configured ? "Configured" : "Not configured"}
                />
              </td>
              <td class="px-3.5 py-2.5">
                <div class="flex flex-wrap gap-1">
                  {#each provider.models as model (model.id)}
                    <span class="rounded border border-line-soft bg-raised px-1.5 py-0.5 text-[9.5px] text-ink-2">
                      {model.name}
                      <span class="font-mono text-ink-4">{Math.round(model.context / 1000)}k</span>
                    </span>
                  {/each}
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </Panel>

  <Panel title="Agent routing" subtitle="Read-only model assignment" padded={false}>
    <div class="divide-y divide-line-soft">
      {#each snapshot.agents as agent (agent.id)}
        {@const model = modelLabel(snapshot, agent)}
        <div class="flex items-center gap-3 px-3.5 py-2.5">
          <Avatar name={agent.name} size="xs" />
          <div class="min-w-0 flex-1">
            <p class="truncate text-[11.5px] text-ink">{agent.name}</p>
            <p class="truncate text-[9.5px] text-ink-4">{agent.role}</p>
          </div>
          <span class="hidden font-mono text-[10px] text-ink-3 sm:block">{model.provider}</span>
          <span class="max-w-[45%] truncate rounded border border-line-soft bg-raised px-1.5 py-0.5 font-mono text-[9.5px] text-ink-2"
            >{model.model}</span
          >
        </div>
      {/each}
    </div>
    <div class="border-t border-line-soft px-3.5 py-2.5 text-[10px] leading-relaxed text-ink-4">
      Per-agent provider and model selection becomes editable with the provider runtime. The snapshot already carries <span
        class="font-mono text-ink-2">providerId</span
      > and <span class="font-mono text-ink-2">modelId</span> for every agent.
    </div>
  </Panel>

  <Panel title="Appearance" subtitle="Dark only in this release">
    <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
      {#each tokens as token (token.name)}
        <div class="rounded-control border border-line-soft bg-canvas p-2">
          <div class="h-8 rounded border border-line-soft" style={`background: ${token.value}`}></div>
          <p class="mt-1.5 text-[10px] text-ink-2">{token.name}</p>
          <p class="font-mono text-[9px] text-ink-4">{token.value}</p>
        </div>
      {/each}
    </div>
    <p class="mt-3 text-[10.5px] leading-relaxed text-ink-3">
      System UI typography, 6 to 10 px radii, no shadows or gradients. Status color is reserved for status. A light theme is
      intentionally out of scope until the dark system settles.
    </p>
  </Panel>

  <Panel title="About">
    <dl class="space-y-2 text-[11px]">
      <div class="flex items-center justify-between gap-4">
        <dt class="text-ink-4">Version</dt>
        <dd class="font-mono text-ink-2">0.1.0</dd>
      </div>
      <div class="flex items-center justify-between gap-4">
        <dt class="text-ink-4">Stack</dt>
        <dd class="text-ink-2">Tauri 2 · Svelte 5 · TypeScript · Vite · Tailwind 4</dd>
      </div>
      <div class="flex items-center justify-between gap-4">
        <dt class="text-ink-4">Repo</dt>
        <dd class="max-w-[70%] truncate font-mono text-[10px] text-ink-2">D:\Projects\agent-bots</dd>
      </div>
      <div class="flex items-center justify-between gap-4">
        <dt class="text-ink-4">State</dt>
        <dd class="text-ink-2">Interface preview · no backend</dd>
      </div>
    </dl>
  </Panel>
</div>
