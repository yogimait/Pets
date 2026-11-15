<script lang="ts">
  import { Command, MessageSquare, PawPrint } from "@lucide/svelte";
  import { app, openOrchestrator, openPets } from "../state/app.svelte.ts";
  import { VIEW_TITLES } from "../nav.ts";
  import { buildMetrics } from "../selectors.ts";
  import { workforceAvailable } from "../workforce.ts";

  const live = workforceAvailable();
  let dataLabel = $derived(live ? "Local data" : "Browser preview");

  let metrics = $derived(app.snapshot ? buildMetrics(app.snapshot) : null);
  let subtitle = $derived(
    metrics
      ? `${metrics.activeAgents} active agents · ${metrics.running} running · ${metrics.blocked} blocked`
      : "",
  );
</script>

<header class="flex h-12 shrink-0 items-center justify-between gap-4 border-b border-line-soft bg-canvas px-4">
  <div class="flex min-w-0 items-center gap-2 whitespace-nowrap text-[11px]">
    <span class="hidden text-ink-4 lg:inline">Agent Bots</span>
    <span class="hidden text-ink-4 lg:inline">/</span>
    <span class="shrink-0 text-ink-2">{VIEW_TITLES[app.view]}</span>
    <span class="ml-2 hidden truncate border-l border-line-soft pl-3 text-[10.5px] text-ink-4 xl:inline">{subtitle}</span>
  </div>

  <div class="flex shrink-0 items-center gap-2">
    <span
      class="hidden items-center gap-1.5 rounded border border-line-soft bg-surface px-2 py-1 text-[10px] text-ink-3 sm:inline-flex"
      title={live
        ? "Workforce data lives in local SQLite."
        : "Browser preview is read-only; open the desktop app for real data."}
    >
      <span class="size-1 rounded-full {live ? 'bg-ok' : 'bg-warn'}" aria-hidden="true"></span>
      {dataLabel}
    </span>

    <button
      onclick={() => (app.paletteOpen = true)}
      class="flex items-center gap-1.5 rounded-control border border-line-soft bg-surface px-2 py-1 text-[11px] text-ink-2 transition-colors hover:bg-hover hover:text-ink"
      aria-label="Open command palette"
    >
      <Command size={12} strokeWidth={1.7} />
      <span class="hidden sm:inline">Command</span>
      <kbd class="ml-1 rounded border border-line-soft px-1 font-mono text-[9.5px] text-ink-4">Ctrl K</kbd>
    </button>

    <button
      onclick={openPets}
      title="Show desktop pets"
      class="hidden items-center gap-1.5 rounded-control border border-line-soft bg-surface px-2 py-1 text-[11px] text-ink-2 transition-colors hover:bg-hover hover:text-ink md:flex"
    >
      <PawPrint size={12} strokeWidth={1.7} />
      Pets
    </button>

    <button
      onclick={openOrchestrator}
      title="Open orchestrator (Alt+`)"
      class="flex items-center gap-1.5 rounded-control border border-line-soft bg-surface px-2 py-1 text-[11px] text-ink-2 transition-colors hover:bg-hover hover:text-ink"
    >
      <MessageSquare size={12} strokeWidth={1.7} />
      Orchestrator
    </button>
  </div>
</header>
