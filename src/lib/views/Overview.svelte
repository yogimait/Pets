<script lang="ts">
  import { MessageSquare } from "@lucide/svelte";
  import TaskRow from "../components/TaskRow.svelte";
  import ActivityRow from "../components/ActivityRow.svelte";
  import { buildMetrics, focusTasks } from "../selectors.ts";
  import { app, openOrchestrator } from "../state/app.svelte.ts";
  import type { Snapshot } from "../types.ts";

  let { snapshot }: { snapshot: Snapshot } = $props();

  let metrics = $derived(buildMetrics(snapshot));
  let tasks = $derived(focusTasks(snapshot).slice(0, 5));
  let events = $derived(snapshot.activity.slice(0, 5));
</script>

<div class="space-y-6">
  <div class="flex flex-wrap items-end justify-between gap-3">
    <div>
      <p class="text-[9.5px] uppercase tracking-[0.16em] text-ink-4">Personal AI workforce</p>
      <h1 class="mt-1 text-[22px] font-medium tracking-[-0.02em] text-ink">Overview</h1>
      <p class="mt-1 text-[11.5px] text-ink-3">
        {metrics.activeAgents} active agents · {metrics.running} running · {metrics.blocked} blocked
      </p>
    </div>
    <button
      onclick={openOrchestrator}
      class="flex items-center gap-1.5 rounded-control border border-accent/30 bg-accent-soft px-3 py-1.5 text-[11px] text-ink transition-colors hover:bg-accent/15"
    >
      <MessageSquare size={13} strokeWidth={1.7} />
      Open orchestrator
    </button>
  </div>

  <div class="grid grid-cols-3 divide-x divide-line-soft overflow-hidden rounded-panel border border-line-soft bg-surface">
    <div class="px-4 py-3">
      <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Agents</p>
      <p class="mt-1 text-[18px] leading-none tabular-nums text-ink">{metrics.activeAgents}</p>
      <p class="mt-1 text-[10px] text-ink-4">of {metrics.totalAgents}</p>
    </div>
    <div class="px-4 py-3">
      <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Running</p>
      <p class="mt-1 text-[18px] leading-none tabular-nums text-ink">{metrics.running}</p>
      <p class="mt-1 text-[10px] text-ink-4">{metrics.review} in review</p>
    </div>
    <div class="px-4 py-3">
      <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Blocked</p>
      <p class="mt-1 text-[18px] leading-none tabular-nums {metrics.blocked > 0 ? 'text-warn' : 'text-ink'}">{metrics.blocked}</p>
      <p class="mt-1 text-[10px] text-ink-4">{metrics.blocked > 0 ? "needs input" : "nothing waiting"}</p>
    </div>
  </div>

  <div class="grid gap-6 xl:grid-cols-2">
    <section>
      <div class="flex items-center justify-between border-b border-line-soft pb-2">
        <h2 class="text-[10px] font-medium uppercase tracking-[0.14em] text-ink-3">Active work</h2>
        <button
          onclick={() => (app.view = "tasks")}
          class="text-[10px] text-ink-4 transition-colors hover:text-ink"
        >
          View all
        </button>      </div>
      <div class="border-x border-b border-line-soft bg-surface">
        {#each tasks as task (task.id)}
          <TaskRow {task} {snapshot} />
        {/each}
        {#if tasks.length === 0}
          <p class="px-3.5 py-6 text-center text-[11px] text-ink-4">
            No open work. Plan a goal or open the task board to start.
          </p>
        {/if}
      </div>
    </section>

    <section>
      <div class="flex items-center justify-between border-b border-line-soft pb-2">
        <h2 class="text-[10px] font-medium uppercase tracking-[0.14em] text-ink-3">Recent activity</h2>
        <button
          onclick={() => (app.view = "activity")}
          class="text-[10px] text-ink-4 transition-colors hover:text-ink"
        >
          All
        </button>
      </div>
      <div class="border-x border-b border-line-soft bg-surface">
        {#each events as event (event.id)}
          <ActivityRow {event} {snapshot} />
        {/each}
        {#if events.length === 0}
          <p class="px-3.5 py-6 text-center text-[11px] text-ink-4">
            No activity yet. Send, plan, or run work to populate the feed.
          </p>
        {/if}
      </div>
    </section>
  </div>

  <p class="text-[10px] text-ink-4">
    {snapshot.agents.length} agents · {snapshot.projects.length} projects · {metrics.memoryEntries} memory entries
  </p>
</div>
