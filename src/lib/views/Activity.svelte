<script lang="ts">
  import Panel from "../components/Panel.svelte";
  import ActivityRow from "../components/ActivityRow.svelte";
  import { activityFiltered } from "../selectors.ts";
  import { dayLabel, plural } from "../format.ts";
  import type { ActivityEvent, ActivityKind, Snapshot } from "../types.ts";

  let { snapshot }: { snapshot: Snapshot } = $props();

  const kinds: { id: ActivityKind | "all"; label: string }[] = [
    { id: "all", label: "All" },
    { id: "delegation", label: "Delegation" },
    { id: "status", label: "Status" },
    { id: "message", label: "Message" },
    { id: "result", label: "Result" },
    { id: "memory", label: "Memory" },
    { id: "system", label: "System" },
  ];

  let kind = $state<ActivityKind | "all">("all");
  let agentId = $state("all");
  let projectId = $state("all");

  let events = $derived(
    activityFiltered(snapshot, {
      kind,
      agentId: agentId === "all" ? null : agentId,
      projectId: projectId === "all" ? null : projectId,
    }),
  );

  let groups = $derived.by(() => {
    const result: { label: string; events: ActivityEvent[] }[] = [];
    for (const event of events) {
      const label = dayLabel(event.at);
      const last = result.at(-1);
      if (last && last.label === label) last.events.push(event);
      else result.push({ label, events: [event] });
    }
    return result;
  });
</script>

<div class="space-y-5">
  <div>
    <h1 class="text-[18px] font-medium tracking-[-0.01em] text-ink">Activity</h1>
    <p class="mt-1 text-[11.5px] text-ink-3">
      {events.length} {plural(events.length, "event")} · unified stream across agents and projects
    </p>
  </div>

  <div class="space-y-2">
    <div class="flex flex-wrap items-center gap-1.5">
      {#each kinds as item (item.id)}
        <button
          onclick={() => (kind = item.id)}
          class="rounded-control border px-2 py-1 text-[10.5px] transition-colors {kind === item.id
            ? 'border-accent/30 bg-accent-soft text-ink'
            : 'border-line-soft bg-surface text-ink-3 hover:bg-hover hover:text-ink'}"
        >
          {item.label}
        </button>
      {/each}
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <select
        bind:value={agentId}
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
      <span class="ml-auto font-mono text-[10px] text-ink-4">{events.length} / {snapshot.activity.length}</span>
    </div>
  </div>

  <div class="space-y-4">
    {#each groups as group (group.label)}
      <section>
        <h2 class="mb-1.5 px-0.5 text-[9.5px] uppercase tracking-[0.14em] text-ink-4">{group.label}</h2>
        <Panel title="" padded={false}>
          {#each group.events as event (event.id)}
            <ActivityRow {event} {snapshot} />
          {/each}
        </Panel>
      </section>
    {/each}

    {#if groups.length === 0}
      <div class="rounded-panel border border-line-soft bg-surface px-4 py-10 text-center text-[11.5px] text-ink-3">
        No activity matches the current filters.
      </div>
    {/if}
  </div>
</div>
