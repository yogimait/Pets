<script lang="ts">
  import { Search } from "@lucide/svelte";
  import Panel from "../components/Panel.svelte";
  import StatusBadge from "../components/StatusBadge.svelte";
  import { agentById, projectById } from "../selectors.ts";
  import { openTask } from "../state/app.svelte.ts";
  import { timeAgo } from "../format.ts";
  import type { Snapshot, TaskStatus } from "../types.ts";

  let { snapshot }: { snapshot: Snapshot } = $props();

  const statuses: { id: TaskStatus | "all"; label: string }[] = [
    { id: "all", label: "All statuses" },
    { id: "running", label: "Running" },
    { id: "review", label: "Review" },
    { id: "blocked", label: "Blocked" },
    { id: "queued", label: "Queued" },
    { id: "done", label: "Done" },
  ];

  const priorityTone: Record<string, string> = {
    low: "text-ink-4",
    normal: "text-ink-3",
    high: "text-warn",
    critical: "text-bad",
  };

  let query = $state("");
  let status = $state<TaskStatus | "all">("all");
  let ownerId = $state("all");
  let projectId = $state("all");

  let filtered = $derived.by(() => {
    const value = query.trim().toLowerCase();
    return snapshot.tasks
      .filter((task) => {
        if (status !== "all" && task.status !== status) return false;
        if (ownerId !== "all" && task.ownerId !== ownerId) return false;
        if (projectId !== "all" && task.projectId !== projectId) return false;
        if (!value) return true;
        return `${task.title} ${task.id} ${task.tags.join(" ")}`.toLowerCase().includes(value);
      })
      .sort((a, b) => b.updatedAt - a.updatedAt);
  });

  let hasFilters = $derived(query.trim() !== "" || status !== "all" || ownerId !== "all" || projectId !== "all");

  function reset() {
    query = "";
    status = "all";
    ownerId = "all";
    projectId = "all";
  }
</script>

<div class="space-y-5">
  <div>
    <h1 class="text-[18px] font-medium tracking-[-0.01em] text-ink">Tasks</h1>
    <p class="mt-1 text-[11.5px] text-ink-3">
      {snapshot.tasks.filter((task) => task.status === "running").length} running ·
      {snapshot.tasks.filter((task) => task.status === "blocked").length} blocked ·
      {snapshot.tasks.filter((task) => task.status === "queued").length} queued ·
      {snapshot.tasks.filter((task) => task.status === "done").length} done
    </p>
  </div>

  <div class="flex flex-wrap items-center gap-2">
    <div class="relative min-w-[180px] flex-1 md:max-w-[260px]">
      <Search size={13} strokeWidth={1.7} class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-4" />
      <input
        bind:value={query}
        placeholder="Filter tasks"
        aria-label="Filter tasks"
        class="h-7 w-full rounded-control border border-line-soft bg-surface pl-7 pr-2 text-[11.5px] text-ink placeholder:text-ink-4 focus:border-accent/40 focus:outline-none"
      />
    </div>

    <select
      bind:value={status}
      aria-label="Filter by status"
      class="h-7 rounded-control border border-line-soft bg-surface px-1.5 text-[11px] text-ink-2 focus:border-accent/40 focus:outline-none"
    >
      {#each statuses as item (item.id)}<option value={item.id}>{item.label}</option>{/each}
    </select>

    <select
      bind:value={ownerId}
      aria-label="Filter by agent"
      class="h-7 rounded-control border border-line-soft bg-surface px-1.5 text-[11px] text-ink-2 focus:border-accent/40 focus:outline-none"
    >
      <option value="all">All agents</option>
      {#each snapshot.agents as agent (agent.id)}<option value={agent.id}>{agent.name}</option>{/each}
    </select>

    <select
      bind:value={projectId}
      aria-label="Filter by project"
      class="h-7 rounded-control border border-line-soft bg-surface px-1.5 text-[11px] text-ink-2 focus:border-accent/40 focus:outline-none"
    >
      <option value="all">All projects</option>
      {#each snapshot.projects as project (project.id)}<option value={project.id}>{project.key} · {project.name}</option>{/each}
    </select>

    {#if hasFilters}
      <button onclick={reset} class="h-7 rounded-control px-2 text-[10.5px] text-ink-3 transition-colors hover:text-ink">
        Reset
      </button>
    {/if}

    <span class="ml-auto font-mono text-[10px] text-ink-4">{filtered.length} / {snapshot.tasks.length}</span>
  </div>

  <Panel title="Task board" padded={false}>
    <div class="overflow-x-auto">
      <table class="w-full border-collapse text-left">
        <thead>
          <tr class="border-b border-line-soft text-[9.5px] uppercase tracking-[0.1em] text-ink-4">
            <th class="px-4 py-2 font-medium">Status</th>
            <th class="px-2 py-2 font-medium">Task</th>
            <th class="hidden px-2 py-2 font-medium min-[1200px]:table-cell">Owner</th>
            <th class="hidden px-2 py-2 font-medium min-[1200px]:table-cell">Project</th>
            <th class="hidden px-2 py-2 font-medium min-[1360px]:table-cell">Priority</th>
            <th class="px-4 py-2 text-right font-medium">Updated</th>
          </tr>
        </thead>
        <tbody>
          {#each filtered as task (task.id)}
            {@const owner = agentById(snapshot, task.ownerId)}
            {@const project = projectById(snapshot, task.projectId)}
            <tr class="border-b border-line-soft transition-colors last:border-0 hover:bg-raised/40">
              <td class="whitespace-nowrap px-4 py-2"><StatusBadge status={task.status} /></td>
              <td class="px-2 py-2">
                <button onclick={() => openTask(task.id)} class="block max-w-full text-left">
                  <span class="block truncate text-[12px] text-ink hover:underline">{task.title}</span>
                  <span class="mt-0.5 block truncate font-mono text-[9px] text-ink-4">
                    {task.id}{#if task.tags.length > 0} · {task.tags.join(" · ")}{/if}
                  </span>
                </button>
              </td>
              <td class="hidden whitespace-nowrap px-2 py-2 text-[11px] text-ink-3 min-[1200px]:table-cell">{owner?.name ?? task.ownerId}</td>
              <td class="hidden whitespace-nowrap px-2 py-2 text-[10.5px] text-ink-3 min-[1200px]:table-cell">{project?.key ?? task.projectId}</td>
              <td class="hidden whitespace-nowrap px-2 py-2 text-[10.5px] {priorityTone[task.priority]} min-[1360px]:table-cell">{task.priority}</td>
              <td class="whitespace-nowrap px-4 py-2 text-right text-[10px] text-ink-4">{timeAgo(task.updatedAt)}</td>
            </tr>
          {/each}
          {#if filtered.length === 0}
            <tr>
              <td colspan="6" class="px-4 py-10 text-center text-[11.5px] text-ink-3">No tasks match the current filters.</td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  </Panel>
</div>
