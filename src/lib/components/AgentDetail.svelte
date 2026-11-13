<script lang="ts">
  import Sheet from "./Sheet.svelte";
  import Section from "./Section.svelte";
  import Avatar from "./Avatar.svelte";
  import StatusBadge from "./StatusBadge.svelte";
  import TaskRow from "./TaskRow.svelte";
  import ActivityRow from "./ActivityRow.svelte";
  import { app, closeSheet, openSheet, openTask } from "../state/app.svelte.ts";
  import { activityFiltered, agentById, modelLabel, projectById, recentTasks, taskById } from "../selectors.ts";
  import { assemblePrompt } from "../prompts.ts";
  import { findAgentPrompt, handoffContract, memoryProposalContract } from "../prompt-files.ts";
  import { timeAgo } from "../format.ts";

  let { id }: { id: string } = $props();

  let agent = $derived(app.snapshot ? agentById(app.snapshot, id) : undefined);
  let model = $derived(agent && app.snapshot ? modelLabel(app.snapshot, agent) : null);
  let currentTask = $derived(
    agent?.currentTaskId && app.snapshot ? taskById(app.snapshot, agent.currentTaskId) : undefined,
  );
  let projects = $derived.by(() => {
    const snapshot = app.snapshot;
    if (!agent || !snapshot) return [];
    return agent.projectIds.map((projectId) => projectById(snapshot, projectId)).filter((project) => !!project);
  });
  let openTasks = $derived(agent && app.snapshot ? recentTasks(app.snapshot, agent.id, 4) : []);
  let events = $derived(agent && app.snapshot ? activityFiltered(app.snapshot, { agentId: agent.id }).slice(0, 4) : []);
  let rolePrompt = $derived(agent ? findAgentPrompt(agent.promptId) : undefined);
  let prompt = $derived.by(() => {
    if (!rolePrompt) return null;
    return assemblePrompt({
      globalPolicy: app.policy?.global.content ?? "",
      projectPolicy: app.policy?.project?.content ?? "",
      rolePrompt: rolePrompt.body,
      handoff: rolePrompt.meta.kind === "specialist" ? handoffContract : null,
      memoryProposal: rolePrompt.meta.kind === "memory" ? memoryProposalContract : null,
    });
  });
</script>

{#if agent && app.snapshot}
  <Sheet title={agent.name} eyebrow={agent.role} onclose={closeSheet}>
    <div class="space-y-5">
      <div class="flex items-start gap-3">
        <Avatar name={agent.name} size="md" accent={agent.role === "Orchestrator"} />
        <div class="min-w-0">
          <div class="flex flex-wrap items-center gap-2">
            <StatusBadge status={agent.status} />
            <span class="text-[10px] text-ink-4">last active {timeAgo(agent.lastActiveAt)}</span>
          </div>
          <p class="mt-2 text-[12px] leading-relaxed text-ink-2">{agent.description}</p>
        </div>
      </div>

      <div class="grid grid-cols-3 gap-2">
        <div class="rounded-control border border-line-soft bg-raised px-2.5 py-2">
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Success</p>
          <p class="mt-1 font-mono text-[13px] text-ink">{agent.successRate}%</p>
        </div>
        <div class="rounded-control border border-line-soft bg-raised px-2.5 py-2">
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Completed</p>
          <p class="mt-1 font-mono text-[13px] text-ink">{agent.tasksCompleted}</p>
        </div>
        <div class="rounded-control border border-line-soft bg-raised px-2.5 py-2">
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Projects</p>
          <p class="mt-1 font-mono text-[13px] text-ink">{agent.projectIds.length}</p>
        </div>
      </div>

      <Section title="Model">
        <div class="rounded-control border border-line-soft bg-raised px-3 py-2.5">
          <p class="text-[12px] text-ink">{model?.model}</p>
          <p class="mt-0.5 text-[10px] text-ink-3">{model?.provider}</p>
        </div>
      </Section>

      {#if currentTask}
        <Section title="Current task">
          <button
            onclick={() => openTask(currentTask!.id)}
            class="w-full rounded-control border border-line-soft bg-raised px-3 py-2.5 text-left transition-colors hover:border-line hover:bg-hover"
          >
            <div class="flex items-center justify-between gap-2">
              <span class="truncate text-[12px] text-ink">{currentTask.title}</span>
              <StatusBadge status={currentTask.status} />
            </div>
            <p class="mt-1 font-mono text-[10px] text-ink-4">{currentTask.id}</p>
          </button>
        </Section>
      {/if}

      <Section title="Capabilities">
        <div class="flex flex-wrap gap-1.5">
          {#each agent.capabilities as capability (capability)}
            <span class="rounded border border-line-soft bg-raised px-1.5 py-0.5 text-[10.5px] text-ink-2">{capability}</span>
          {/each}
        </div>
      </Section>

      <Section title="Instructions">
        <pre
          class="max-h-52 overflow-y-auto whitespace-pre-wrap rounded-control border border-line-soft bg-canvas p-3 font-mono text-[10.5px] leading-relaxed text-ink-2">{agent.instructions}</pre>
      </Section>

      <Section title="Assembled prompt">
        {#if prompt && rolePrompt}
          <p class="mb-2 font-mono text-[9px] text-ink-4">
            {rolePrompt.meta.id}.md · global {app.policy?.global.hash || "missing"} · project {app.policy?.project?.hash || "none"}
          </p>
          <pre
            class="max-h-64 overflow-y-auto whitespace-pre-wrap rounded-control border border-line-soft bg-canvas p-3 font-mono text-[10px] leading-relaxed text-ink-2">{prompt}</pre>
        {:else}
          <p class="text-[11px] text-ink-4">No prompt file is registered for this agent.</p>
        {/if}
      </Section>

      <Section title="Projects">
        <div class="flex flex-wrap gap-1.5">
          {#each projects as project (project!.id)}
            <button
              onclick={() => openSheet("project", project!.id)}
              class="rounded-control border border-line-soft bg-raised px-2 py-1 text-[10.5px] text-ink-2 transition-colors hover:border-line hover:text-ink"
            >
              {project!.key} · {project!.name}
            </button>
          {/each}
        </div>
      </Section>

      {#if openTasks.length > 0}
        <Section title="Open tasks">
          <div class="overflow-hidden rounded-control border border-line-soft bg-surface">
            {#each openTasks as task (task.id)}
              <TaskRow {task} snapshot={app.snapshot} />
            {/each}
          </div>
        </Section>
      {/if}

      {#if events.length > 0}
        <Section title="Recent activity">
          <div class="overflow-hidden rounded-control border border-line-soft bg-surface">
            {#each events as event (event.id)}
              <ActivityRow {event} snapshot={app.snapshot} />
            {/each}
          </div>
        </Section>
      {/if}
    </div>
  </Sheet>
{/if}
