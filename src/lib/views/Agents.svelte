<script lang="ts">
  import Avatar from "../components/Avatar.svelte";
  import StatusBadge from "../components/StatusBadge.svelte";
  import { opencodeAgentId, taskById } from "../selectors.ts";
  import { app, openSheet } from "../state/app.svelte.ts";
  import { timeAgo } from "../format.ts";
  import type { AgentStatus, Snapshot } from "../types.ts";

  let { snapshot }: { snapshot: Snapshot } = $props();

  const filters: { id: AgentStatus | "all"; label: string }[] = [
    { id: "all", label: "All" },
    { id: "active", label: "Active" },
    { id: "idle", label: "Idle" },
    { id: "paused", label: "Paused" },
    { id: "offline", label: "Offline" },
  ];

  let filter = $state<AgentStatus | "all">("all");
  let agents = $derived(filter === "all" ? snapshot.agents : snapshot.agents.filter((agent) => agent.status === filter));

  function runtimeAgent(agentId: string): string {
    const agent = snapshot.agents.find((candidate) => candidate.id === agentId);
    return agent ? opencodeAgentId(agent) : "unassigned";
  }

  function currentTask(taskId: string | null): string {
    if (!taskId) return "No claimed task";
    return taskById(snapshot, taskId)?.title ?? "No claimed task";
  }
</script>

<div class="space-y-5">
  <div>
    <h1 class="text-[18px] font-medium tracking-[-0.01em] text-ink">Agents</h1>
    <p class="mt-1 text-[11.5px] text-ink-3">
      {snapshot.agents.filter((agent) => agent.status === "active").length} active · {snapshot.agents.length} configured · one orchestrator, one memory agent
    </p>
  </div>

  <div class="flex flex-wrap items-center gap-1.5">
    {#each filters as item (item.id)}
      {@const count = item.id === "all" ? snapshot.agents.length : snapshot.agents.filter((agent) => agent.status === item.id).length}
      <button
        onclick={() => (filter = item.id)}
        class="rounded-control border px-2 py-1 text-[10.5px] transition-colors {filter === item.id
          ? 'border-accent/30 bg-accent-soft text-ink'
          : 'border-line-soft bg-surface text-ink-3 hover:bg-hover hover:text-ink'}"
      >
        {item.label}
        <span class="ml-1 font-mono text-[9.5px] text-ink-4">{count}</span>
      </button>
    {/each}
  </div>

  <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
    {#each agents as agent (agent.id)}
      <button
        onclick={() => openSheet("agent", agent.id)}
        class="flex flex-col rounded-panel border border-line-soft bg-surface p-3.5 text-left transition-colors hover:border-line hover:bg-raised/40"
      >
        <div class="flex items-start justify-between gap-2">
          <div class="flex min-w-0 items-center gap-2.5">
            <Avatar name={agent.name} agentId={agent.id} size="md" accent={agent.role === "Orchestrator"} />
            <div class="min-w-0">
              <p class="truncate text-[13px] font-medium text-ink">{agent.name}</p>
              <p class="truncate text-[10.5px] text-ink-3">{agent.role}</p>
            </div>
          </div>
          <StatusBadge status={agent.id === "dreamer" && app.dreamerRunning ? "active" : agent.status} />
        </div>

        <p class="mt-3 line-clamp-2 min-h-[32px] text-[11px] leading-relaxed text-ink-3">{agent.description}</p>

        <p class="mt-2 truncate text-[10.5px] text-ink-3">
          <span class="text-ink-4">on</span>
          {currentTask(agent.currentTaskId)}
        </p>

        <div class="mt-auto flex items-center justify-between gap-2 border-t border-line-soft pt-2.5 text-[9.5px] text-ink-4">
          <span class="truncate font-mono">OpenCode: {runtimeAgent(agent.id)}</span>
          <span class="shrink-0">{agent.successRate}% · {agent.tasksCompleted} done</span>
        </div>
        <p class="mt-1 text-[9.5px] text-ink-4">{agent.lastActiveAt ? `last active ${timeAgo(agent.lastActiveAt)}` : "no activity yet"}</p>
      </button>
    {/each}
  </div>

  {#if agents.length === 0}
    <div class="rounded-panel border border-line-soft bg-surface px-4 py-10 text-center text-[11.5px] text-ink-3">
      No agents match this filter.
    </div>
  {/if}
</div>
