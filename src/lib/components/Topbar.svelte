<script lang="ts">
  import { Bell, Command, MessageSquare, PawPrint } from "@lucide/svelte";
  import { app, openOrchestrator, openTask, togglePets } from "../state/app.svelte.ts";
  import { VIEW_TITLES } from "../nav.ts";
  import { agentById, buildMetrics, projectById } from "../selectors.ts";
  import { workforceAvailable } from "../workforce.ts";
  import type { Task } from "../types.ts";

  const live = workforceAvailable();
  let dataLabel = $derived(live ? "Local data" : "Browser preview");
  let notificationsOpen = $state(false);

  let attention = $derived.by(() => {
    if (!app.snapshot) return [];
    return app.snapshot.tasks
      .filter((task) => task.permission || task.status === "review" || task.status === "blocked")
      .sort((a, b) => {
        const priority = (task: Task) => (task.permission ? 0 : task.status === "review" ? 1 : 2);
        return priority(a) - priority(b) || b.updatedAt - a.updatedAt;
      });
  });
  let attentionCount = $derived(attention.length);

  function attentionLabel(task: Task): string {
    if (task.permission) return "Permission needed";
    if (task.status === "review") return "Review result";
    return "Blocked";
  }

  function openAttention(task: Task) {
    notificationsOpen = false;
    openTask(task.id);
  }

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

    <div class="relative">
      <button
        onclick={() => (notificationsOpen = !notificationsOpen)}
        aria-label={`Notifications (${attentionCount})`}
        aria-expanded={notificationsOpen}
        aria-haspopup="true"
        class="relative flex items-center gap-1.5 rounded-control border border-line-soft bg-surface px-2 py-1 text-[11px] text-ink-2 transition-colors hover:bg-hover hover:text-ink"
      >
        <Bell size={12} strokeWidth={1.7} />
        <span class="hidden sm:inline">Alerts</span>
        {#if attentionCount > 0}
          <span class="grid min-w-4 place-items-center rounded-full bg-warn px-1 text-[9px] font-medium text-canvas">
            {attentionCount > 9 ? "9+" : attentionCount}
          </span>
        {/if}
      </button>

      {#if notificationsOpen}
        <div class="absolute right-0 top-[calc(100%+8px)] z-50 w-[320px] overflow-hidden rounded-panel border border-line bg-surface">
          <div class="flex items-center justify-between border-b border-line-soft px-3 py-2">
            <span class="text-[10px] uppercase tracking-[0.12em] text-ink-3">Needs attention</span>
            <span class="font-mono text-[10px] text-ink-4">{attentionCount}</span>
          </div>
          {#if attention.length > 0}
            <div class="max-h-80 overflow-y-auto">
              {#each attention as task (task.id)}
                {@const project = app.snapshot ? projectById(app.snapshot, task.projectId) : undefined}
                {@const owner = app.snapshot ? agentById(app.snapshot, task.ownerId) : undefined}
                <button
                  onclick={() => openAttention(task)}
                  class="block w-full border-b border-line-soft px-3 py-2.5 text-left transition-colors last:border-0 hover:bg-raised"
                >
                  <span class="block text-[9.5px] uppercase tracking-[0.1em] {task.permission ? 'text-warn' : task.status === 'review' ? 'text-ok' : 'text-bad'}">
                    {attentionLabel(task)}
                  </span>
                  <span class="mt-0.5 block truncate text-[11.5px] text-ink">{task.title}</span>
                  <span class="mt-0.5 block font-mono text-[9.5px] text-ink-4">
                    {project?.key ?? task.projectId} A� {owner?.name ?? task.ownerId}
                  </span>
                </button>
              {/each}
            </div>
          {:else}
            <p class="px-3 py-6 text-center text-[11px] text-ink-3">No approvals or reviews waiting.</p>
          {/if}
        </div>
      {/if}
    </div>

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
      onclick={togglePets}
      title={app.petsVisible ? "Hide desktop pets" : "Show desktop pets"}
      aria-pressed={app.petsVisible}
      class="hidden items-center gap-1.5 rounded-control border border-line-soft bg-surface px-2 py-1 text-[11px] text-ink-2 transition-colors hover:bg-hover hover:text-ink md:flex"
    >
      <PawPrint size={12} strokeWidth={1.7} />
      {app.petsVisible ? "Pets" : "Show pets"}
    </button>

    <button
      onclick={openOrchestrator}
      title="Open Micheal · Orchestrator (Alt+`)"
      class="flex items-center gap-1.5 rounded-control border border-line-soft bg-surface px-2 py-1 text-[11px] text-ink-2 transition-colors hover:bg-hover hover:text-ink"
    >
      <MessageSquare size={12} strokeWidth={1.7} />
      Micheal
    </button>
  </div>
</header>
