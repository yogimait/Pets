<script lang="ts">
  import Sheet from "./Sheet.svelte";
  import Section from "./Section.svelte";
  import Avatar from "./Avatar.svelte";
  import StatusBadge from "./StatusBadge.svelte";
  import TaskRow from "./TaskRow.svelte";
  import { app, closeSheet, openSheet } from "../state/app.svelte.ts";
  import { agentsForProject, artifactsFor, memoryFiltered, projectById, projectStats, tasksByProject } from "../selectors.ts";
  import { timeAgo } from "../format.ts";

  let { id }: { id: string } = $props();

  let project = $derived(app.snapshot ? projectById(app.snapshot, id) : undefined);
  let stats = $derived(project && app.snapshot ? projectStats(app.snapshot, project.id) : null);
  let tasks = $derived(project && app.snapshot ? tasksByProject(app.snapshot, project.id) : []);
  let agents = $derived(project && app.snapshot ? agentsForProject(app.snapshot, project.id) : []);
  let artifacts = $derived(project && app.snapshot ? artifactsFor(app.snapshot, project.id) : []);
  let memories = $derived(project && app.snapshot ? memoryFiltered(app.snapshot, { projectId: project.id }) : []);
</script>

{#if project && app.snapshot && stats}
  <Sheet title={project.name} eyebrow={`${project.key} · project`} onclose={closeSheet}>
    <div class="space-y-5">
      <div>
        <div class="flex flex-wrap items-center gap-2">
          <StatusBadge status={project.status} />
          <span class="text-[10px] text-ink-4">created {timeAgo(project.createdAt)}</span>
        </div>
        <p class="mt-2 text-[12px] leading-relaxed text-ink-2">{project.description}</p>
      </div>

      <div class="grid grid-cols-3 gap-2">
        <div class="rounded-control border border-line-soft bg-raised px-2.5 py-2">
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Tasks</p>
          <p class="mt-1 font-mono text-[13px] text-ink">{stats.done}/{stats.total}</p>
        </div>
        <div class="rounded-control border border-line-soft bg-raised px-2.5 py-2">
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Running</p>
          <p class="mt-1 font-mono text-[13px] text-ink">{stats.running}</p>
        </div>
        <div class="rounded-control border border-line-soft bg-raised px-2.5 py-2">
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Memory</p>
          <p class="mt-1 font-mono text-[13px] text-ink">{memories.length}</p>
        </div>
      </div>

      <Section title="Context">
        <p class="whitespace-pre-line text-[12px] leading-relaxed text-ink-2">{project.context}</p>
      </Section>

      <Section title="Instructions">
        <p class="whitespace-pre-line rounded-control border border-line-soft bg-canvas p-3 text-[11px] leading-relaxed text-ink-2"
          >{project.instructions}</p
        >
      </Section>

      <Section title="Repo">
        <p class="break-all rounded-control border border-line-soft bg-raised px-3 py-2 font-mono text-[10.5px] text-ink-2">
          {project.repoPath}
        </p>
      </Section>

      <Section title="Agents">
        <div class="space-y-1.5">
          {#each agents as agent (agent.id)}
            <button
              onclick={() => openSheet("agent", agent.id)}
              class="flex w-full items-center gap-2.5 rounded-control border border-line-soft bg-surface px-2.5 py-1.5 text-left transition-colors hover:bg-raised"
            >
              <Avatar name={agent.name} size="xs" />
              <span class="min-w-0 flex-1 truncate text-[11.5px] text-ink">{agent.name}</span>
              <span class="text-[10px] text-ink-4">{agent.role}</span>
            </button>
          {/each}
        </div>
      </Section>

      <Section title={`Tasks · ${tasks.length}`}>
        <div class="overflow-hidden rounded-control border border-line-soft bg-surface">
          {#each tasks as task (task.id)}
            <TaskRow {task} snapshot={app.snapshot} />
          {/each}
          {#if tasks.length === 0}
            <p class="px-3 py-4 text-[11px] text-ink-4">No tasks yet.</p>
          {/if}
        </div>
      </Section>

      <Section title="Artifacts">
        <div class="space-y-1">
          {#each artifacts as artifact (artifact.id)}
            <div class="flex items-center gap-2 rounded-control border border-line-soft bg-surface px-2.5 py-1.5">
              <span class="rounded border border-line-soft px-1 font-mono text-[9px] uppercase text-ink-4">{artifact.kind}</span>
              <span class="min-w-0 flex-1 truncate text-[11px] text-ink-2">{artifact.name}</span>
              <span class="max-w-[45%] truncate font-mono text-[9.5px] text-ink-4">{artifact.path}</span>
            </div>
          {/each}
          {#if artifacts.length === 0}
            <p class="text-[11px] text-ink-4">No artifacts recorded.</p>
          {/if}
        </div>
      </Section>

      <Section title="Recent memory">
        <div class="space-y-1">
          {#each memories.slice(0, 3) as entry (entry.id)}
            <button
              onclick={() => openSheet("memory", entry.id)}
              class="w-full rounded-control border border-line-soft bg-surface px-2.5 py-1.5 text-left transition-colors hover:bg-raised"
            >
              <p class="truncate text-[11.5px] text-ink">{entry.title}</p>
              <p class="mt-0.5 text-[9.5px] text-ink-4">{entry.kind} · {timeAgo(entry.updatedAt)}</p>
            </button>
          {/each}
          {#if memories.length === 0}
            <p class="text-[11px] text-ink-4">No memory entries for this project.</p>
          {/if}
        </div>
      </Section>
    </div>
  </Sheet>
{/if}
