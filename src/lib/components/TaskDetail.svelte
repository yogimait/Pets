<script lang="ts">
  import { Check } from "@lucide/svelte";
  import Sheet from "./Sheet.svelte";
  import Section from "./Section.svelte";
  import Avatar from "./Avatar.svelte";
  import StatusBadge from "./StatusBadge.svelte";
  import { app, closeSheet, openSheet, openTask } from "../state/app.svelte.ts";
  import { agentById, projectById, taskById, taskStepProgress } from "../selectors.ts";
  import { fullTimestamp, timeAgo } from "../format.ts";

  let { id }: { id: string } = $props();

  let task = $derived(app.snapshot ? taskById(app.snapshot, id) : undefined);
  let project = $derived(task && app.snapshot ? projectById(app.snapshot, task.projectId) : undefined);
  let owner = $derived(task && app.snapshot ? agentById(app.snapshot, task.ownerId) : undefined);
  let progress = $derived(task ? taskStepProgress(task) : 0);

  const priorityTone: Record<string, string> = {
    low: "text-ink-4",
    normal: "text-ink-3",
    high: "text-warn",
    critical: "text-bad",
  };

  function senderName(senderId: string): string {
    if (!app.snapshot) return senderId;
    return agentById(app.snapshot, senderId)?.name ?? senderId;
  }
</script>

{#if task}
  <Sheet title={task.title} eyebrow={`${project?.key ?? task.projectId} · ${task.id}`} onclose={closeSheet}>
    <div class="space-y-5">
      <div class="flex flex-wrap items-center gap-2">
        <StatusBadge status={task.status} />
        <span class="rounded border border-line-soft bg-raised px-1.5 py-[1px] text-[10.5px] {priorityTone[task.priority]}"
          >{task.priority} priority</span
        >
        <span class="text-[10px] text-ink-4">updated {timeAgo(task.updatedAt)}</span>
      </div>

      <p class="text-[12px] leading-relaxed text-ink-2">{task.description}</p>

      <div class="grid grid-cols-2 gap-2">
        <button
          onclick={() => owner && openSheet("agent", owner.id)}
          class="rounded-control border border-line-soft bg-raised px-2.5 py-2 text-left transition-colors hover:border-line hover:bg-hover"
        >
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Owner</p>
          <div class="mt-1.5 flex items-center gap-1.5">
            <Avatar name={owner?.name ?? task.ownerId} size="xs" />
            <span class="truncate text-[11.5px] text-ink">{owner?.name ?? task.ownerId}</span>
          </div>
        </button>
        <button
          onclick={() => project && openSheet("project", project.id)}
          class="rounded-control border border-line-soft bg-raised px-2.5 py-2 text-left transition-colors hover:border-line hover:bg-hover"
        >
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Project</p>
          <span class="mt-1.5 block truncate text-[11.5px] text-ink">{project?.name ?? task.projectId}</span>
        </button>
      </div>

      {#if task.steps.length > 0}
        <Section title={`Steps · ${task.steps.filter((step) => step.done).length}/${task.steps.length}`}>
          <div class="mb-2 h-1 overflow-hidden rounded-full bg-raised">
            <div class="h-full rounded-full bg-accent/70" style={`width: ${Math.round(progress * 100)}%`}></div>
          </div>
          <ul class="space-y-1.5">
            {#each task.steps as step (step.label)}
              <li class="flex items-start gap-2 text-[11.5px] {step.done ? 'text-ink-3' : 'text-ink-2'}">
                <span
                  class="mt-[1px] grid size-4 shrink-0 place-items-center rounded-full border {step.done
                    ? 'border-ok/40 text-ok'
                    : 'border-line text-transparent'}"
                >
                  <Check size={10} strokeWidth={2.2} />
                </span>
                <span class={step.done ? "line-through decoration-line" : ""}>{step.label}</span>
              </li>
            {/each}
          </ul>
        </Section>
      {/if}

      {#if task.notes}
        <Section title="Notes">
          <p class="rounded-control border border-line-soft bg-canvas p-3 text-[11.5px] leading-relaxed text-ink-2">{task.notes}</p>
        </Section>
      {/if}

      {#if task.tags.length > 0}
        <Section title="Tags">
          <div class="flex flex-wrap gap-1.5">
            {#each task.tags as tag (tag)}
              <span class="rounded border border-line-soft bg-raised px-1.5 py-0.5 font-mono text-[9.5px] text-ink-3">{tag}</span>
            {/each}
          </div>
        </Section>
      {/if}

      <Section title={`Thread · ${task.messages.length}`}>
        <div class="space-y-2.5">
          {#each task.messages as message (message.id)}
            <div class="flex gap-2">
              <Avatar name={senderName(message.senderId)} size="xs" />
              <div class="min-w-0 flex-1">
                <div class="flex items-baseline gap-2">
                  <span class="text-[10.5px] text-ink-2">{senderName(message.senderId)}</span>
                  <span class="font-mono text-[9px] text-ink-4">{timeAgo(message.at)}</span>
                </div>
                <p class="mt-0.5 text-[11.5px] leading-relaxed text-ink-2">{message.text}</p>
              </div>
            </div>
          {/each}
          {#if task.messages.length === 0}
            <p class="text-[11px] text-ink-4">No messages on this task.</p>
          {/if}
        </div>
      </Section>

      <div class="flex items-center justify-between border-t border-line-soft pt-3 text-[9.5px] text-ink-4">
        <span>created {fullTimestamp(task.createdAt)}</span>
        <span>updated {fullTimestamp(task.updatedAt)}</span>
      </div>

      <button
        onclick={() => openTask(task.id)}
        class="w-full rounded-control border border-line-soft bg-raised px-3 py-2 text-[11px] text-ink-2 transition-colors hover:bg-hover hover:text-ink"
      >
        Open in task board
      </button>
    </div>
  </Sheet>
{/if}
