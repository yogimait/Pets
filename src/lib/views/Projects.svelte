<script lang="ts">
  import Panel from "../components/Panel.svelte";
  import Avatar from "../components/Avatar.svelte";
  import StatusBadge from "../components/StatusBadge.svelte";
  import { agentsForProject, projectStats, tasksByProject } from "../selectors.ts";
  import { openSheet } from "../state/app.svelte.ts";
  import { percent, timeAgo } from "../format.ts";
  import type { Snapshot } from "../types.ts";

  let { snapshot }: { snapshot: Snapshot } = $props();
</script>

<div class="space-y-5">
  <div>
    <h1 class="text-[18px] font-medium tracking-[-0.01em] text-ink">Projects</h1>
    <p class="mt-1 text-[11.5px] text-ink-3">
      {snapshot.projects.filter((project) => project.status === "active").length} active · {snapshot.projects.length} workspaces · each keeps its own context and instructions
    </p>
  </div>

  <Panel title="Workspaces" subtitle="Context, agents, and task progress" padded={false}>
    {#each snapshot.projects as project (project.id)}
      {@const stats = projectStats(snapshot, project.id)}
      {@const projectTasks = tasksByProject(snapshot, project.id)}
      {@const projectAgents = agentsForProject(snapshot, project.id)}
      {@const updated = Math.max(...projectTasks.map((task) => task.updatedAt), project.createdAt)}
      <button
        onclick={() => openSheet("project", project.id)}
        class="grid w-full grid-cols-[44px_minmax(0,1fr)] items-start gap-4 border-b border-line-soft px-4 py-3.5 text-left transition-colors last:border-b-0 hover:bg-raised/40 md:grid-cols-[44px_minmax(0,1fr)_auto]"
      >
        <span class="pt-0.5 font-mono text-[10px] tracking-wide text-ink-4">{project.key}</span>

        <div class="min-w-0">
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-[12.5px] font-medium text-ink">{project.name}</span>
            <StatusBadge status={project.status} />
          </div>
          <p class="mt-0.5 truncate text-[10.5px] text-ink-3">{project.description}</p>
          <p class="mt-1.5 truncate font-mono text-[9.5px] text-ink-4">{project.repoPath}</p>
        </div>

        <div class="col-span-2 flex flex-wrap items-center gap-5 md:col-span-1 md:justify-end">
          <div class="flex -space-x-1.5" aria-label={`${projectAgents.length} agents`}>
            {#each projectAgents.slice(0, 4) as agent (agent.id)}
              <span class="rounded-control ring-2 ring-surface">
                <Avatar name={agent.name} size="xs" />
              </span>
            {/each}
            {#if projectAgents.length > 4}
              <span class="grid size-5 place-items-center rounded-control border border-line-soft bg-raised font-mono text-[8.5px] text-ink-4"
                >+{projectAgents.length - 4}</span
              >
            {/if}
          </div>

          <div class="w-28">
            <div class="flex items-center justify-between text-[9px] text-ink-4">
              <span class="font-mono">{stats.done}/{stats.total}</span>
              <span class="font-mono">{percent(stats.progress)}</span>
            </div>
            <div class="mt-1 h-1 overflow-hidden rounded-full bg-raised">
              <div class="h-full rounded-full bg-accent/70" style={`width: ${Math.round(stats.progress * 100)}%`}></div>
            </div>
          </div>

          <span class="w-16 text-right text-[9.5px] text-ink-4">{timeAgo(updated)}</span>
        </div>
      </button>
    {/each}
  </Panel>

  <p class="text-[10px] text-ink-4">
    Progress counts completed tasks only. A project with no tasks shows an empty bar by design.
  </p>
</div>
