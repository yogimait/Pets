<script lang="ts">
  import { MessageSquare } from "@lucide/svelte";
  import Metric from "../components/Metric.svelte";
  import Panel from "../components/Panel.svelte";
  import Avatar from "../components/Avatar.svelte";
  import StatusBadge from "../components/StatusBadge.svelte";
  import TaskRow from "../components/TaskRow.svelte";
  import ActivityRow from "../components/ActivityRow.svelte";
  import { buildMetrics, focusTasks, taskById } from "../selectors.ts";
  import { timeAgo } from "../format.ts";
  import { app, openOrchestrator, openSheet } from "../state/app.svelte.ts";
  import type { Snapshot } from "../types.ts";

  let { snapshot }: { snapshot: Snapshot } = $props();

  let metrics = $derived(buildMetrics(snapshot));
  let tasks = $derived(focusTasks(snapshot).slice(0, 5));
  let events = $derived(snapshot.activity.slice(0, 6));
  let latestMemory = $derived([...snapshot.memory].sort((a, b) => b.updatedAt - a.updatedAt).slice(0, 3));
  let openCount = $derived(metrics.running + metrics.review + metrics.queued + metrics.blocked);

  function currentTaskTitle(taskId: string | null): string {
    if (!taskId) return "No claimed task";
    return taskById(snapshot, taskId)?.title ?? "No claimed task";
  }
</script>

<div class="space-y-5">
  <div class="flex flex-wrap items-end justify-between gap-3">
    <div>
      <h1 class="text-[18px] font-medium tracking-[-0.01em] text-ink">Workforce overview</h1>
      <p class="mt-1 text-[11.5px] text-ink-3">
        {metrics.activeAgents} of {metrics.totalAgents} agents active · {metrics.activeProjects} projects · snapshot from local mock data
      </p>
    </div>
    <button
      onclick={openOrchestrator}
      class="flex items-center gap-1.5 rounded-control border border-line-soft bg-surface px-2.5 py-1.5 text-[11px] text-ink-2 transition-colors hover:bg-hover hover:text-ink"
    >
      <MessageSquare size={13} strokeWidth={1.7} />
      Open orchestrator
    </button>
  </div>

  <div class="grid grid-cols-2 gap-3 xl:grid-cols-4">
    <Metric label="Active agents" value={`${metrics.activeAgents} / ${metrics.totalAgents}`} hint="Orchestrator online" tone="ok" />
    <Metric label="Tasks running" value={metrics.running} hint={`${metrics.review} waiting in review`} />
    <Metric label="Blocked" value={metrics.blocked} hint="Needs a decision" tone={metrics.blocked > 0 ? "warn" : "ink"} />
    <Metric label="Memory entries" value={metrics.memoryEntries} hint={`${snapshot.projects.length} projects covered`} />
  </div>

  <div class="grid items-start gap-4 xl:grid-cols-[minmax(0,1.35fr)_minmax(0,1fr)]">
    <Panel title="Task board" subtitle={`${openCount} open`} padded={false}>
      {#snippet action()}
        <button
          onclick={() => (app.view = "tasks")}
          class="rounded border border-line-soft px-1.5 py-0.5 text-[10px] text-ink-3 transition-colors hover:text-ink"
        >
          View all
        </button>
      {/snippet}
      {#each tasks as task (task.id)}
        <TaskRow {task} {snapshot} />
      {/each}
    </Panel>

    <Panel title="Recent activity" subtitle="Across the workforce" padded={false}>
      {#snippet action()}
        <button
          onclick={() => (app.view = "activity")}
          class="rounded border border-line-soft px-1.5 py-0.5 text-[10px] text-ink-3 transition-colors hover:text-ink"
        >
          View all
        </button>
      {/snippet}
      {#each events as event (event.id)}
        <ActivityRow {event} {snapshot} />
      {/each}
    </Panel>
  </div>

  <div class="grid items-start gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
    <Panel title="Agents" subtitle="Status and current work" padded={false}>
      {#each snapshot.agents as agent (agent.id)}
        <button
          onclick={() => openSheet("agent", agent.id)}
          class="flex w-full items-center gap-2.5 border-b border-line-soft px-3.5 py-2 text-left transition-colors last:border-b-0 hover:bg-raised/50"
        >
          <Avatar name={agent.name} size="sm" accent={agent.role === "Orchestrator"} />
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <span class="text-[11.5px] text-ink">{agent.name}</span>
              <span class="truncate text-[9.5px] text-ink-4">{agent.role}</span>
            </div>
            <p class="truncate text-[10px] text-ink-3">{currentTaskTitle(agent.currentTaskId)}</p>
          </div>
          <StatusBadge status={agent.status} />
        </button>
      {/each}
    </Panel>

    <Panel title="Dreamer" subtitle="Persistent memory" padded={false}>
      {#snippet action()}
        <button
          onclick={() => (app.view = "memory")}
          class="rounded border border-line-soft px-1.5 py-0.5 text-[10px] text-ink-3 transition-colors hover:text-ink"
        >
          Open memory
        </button>
      {/snippet}
      <div class="border-b border-line-soft px-3.5 py-2.5">
        <p class="text-[11px] leading-relaxed text-ink-3">
          {metrics.memoryEntries} entries across {snapshot.projects.length} projects. Last consolidation
          {timeAgo(Math.max(...snapshot.memory.map((entry) => entry.updatedAt)))}.
        </p>
      </div>
      {#each latestMemory as entry (entry.id)}
        <button
          onclick={() => openSheet("memory", entry.id)}
          class="flex w-full items-center gap-2.5 border-b border-line-soft px-3.5 py-2 text-left transition-colors last:border-b-0 hover:bg-raised/50"
        >
          <span class="rounded border border-line-soft px-1 py-[1px] font-mono text-[9px] text-ink-4">{entry.kind}</span>
          <span class="min-w-0 flex-1 truncate text-[11px] text-ink-2">{entry.title}</span>
          <span class="shrink-0 text-[9.5px] text-ink-4">{timeAgo(entry.updatedAt)}</span>
        </button>
      {/each}
    </Panel>
  </div>

  <div class="flex flex-wrap items-center gap-1.5 pb-1">
    <span class="text-[9.5px] uppercase tracking-[0.14em] text-ink-4">Projects</span>
    {#each snapshot.projects as project (project.id)}
      <button
        onclick={() => openSheet("project", project.id)}
        class="rounded border border-line-soft bg-surface px-1.5 py-0.5 text-[10px] text-ink-3 transition-colors hover:bg-raised hover:text-ink"
      >
        {project.key} · {project.name}
      </button>
    {/each}
  </div>
</div>
