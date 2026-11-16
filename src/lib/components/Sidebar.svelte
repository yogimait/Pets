<script lang="ts">
  import { Brain } from "@lucide/svelte";
  import { NAV, type View } from "../nav.ts";
  import { app, go } from "../state/app.svelte.ts";
  import { buildMetrics } from "../selectors.ts";
  import { timeAgo } from "../format.ts";

  let metrics = $derived(app.snapshot ? buildMetrics(app.snapshot) : null);
  let lastMemory = $derived(
    app.snapshot && app.snapshot.memory.length > 0
      ? Math.max(...app.snapshot.memory.map((entry) => entry.updatedAt))
      : 0,
  );
  let projectCount = $derived(app.snapshot?.projects.length ?? 0);

  function navigate(view: View) {
    go(view);
  }
</script>

<nav class="flex w-[220px] shrink-0 flex-col border-r border-line-soft bg-canvas" aria-label="Primary">
  <div class="flex items-center gap-2.5 px-4 pb-4 pt-4">
    <div
      class="grid size-7 place-items-center rounded-control border border-line bg-raised font-mono text-[10px] text-accent"
      aria-hidden="true"
    >
      AB
    </div>
    <div class="min-w-0">
      <p class="text-[12.5px] font-medium leading-tight text-ink">Agent Bots</p>
      <p class="text-[10.5px] leading-tight text-ink-3">Workforce control</p>
    </div>
  </div>

  <p class="px-4 pb-1.5 text-[9.5px] uppercase tracking-[0.16em] text-ink-4">Workspace</p>
  <div class="flex flex-col gap-0.5 px-3">
    {#each NAV as item (item.id)}
      {@const Icon = item.icon}
      <button
        onclick={() => navigate(item.id)}
        class="group relative flex w-full items-center gap-2.5 rounded-control px-2.5 py-[7px] text-left text-[12.5px] transition-colors {app.view ===
        item.id
          ? 'bg-raised text-ink'
          : 'text-ink-2 hover:bg-raised/60 hover:text-ink'}"
        aria-current={app.view === item.id ? "page" : undefined}
      >
        {#if app.view === item.id}
          <span class="absolute inset-y-1.5 left-0 w-[2px] rounded-full bg-accent" aria-hidden="true"></span>
        {/if}
        <Icon size={15} strokeWidth={1.6} class={app.view === item.id ? "text-accent" : "text-ink-3"} />
        <span class="flex-1">{item.label}</span>
        <span class="text-[10px] text-ink-4 opacity-0 transition-opacity group-hover:opacity-100"
          >{item.hint}</span
        >
      </button>
    {/each}
  </div>

  <div class="mt-auto space-y-3 p-3">
    <div class="rounded-panel border border-line-soft bg-surface p-3">
      <div class="flex items-center justify-between">
        <span class="flex items-center gap-1.5 text-[11px] text-ink-2">
          <Brain size={12} strokeWidth={1.6} class="text-ink-3" />
          Holly
        </span>
        <span class="text-[10px] text-ink-4">{app.dreamerRunning ? "running" : "idle"}</span>
      </div>
      <p class="mt-1.5 font-mono text-[10.5px] text-ink-3">{metrics?.memoryEntries ?? 0} entries · {projectCount} projects</p>
      <p class="mt-0.5 text-[10px] text-ink-4">last write {lastMemory ? timeAgo(lastMemory) : "none"}</p>
    </div>
    <div class="flex items-center justify-between px-1 text-[10px] text-ink-4">
      <span>v0.1.0</span>
      <span>local build</span>
    </div>
  </div>
</nav>
