<script lang="ts">
  import Sheet from "./Sheet.svelte";
  import Section from "./Section.svelte";
  import StatusBadge from "./StatusBadge.svelte";
  import { app, closeSheet, openTask } from "../state/app.svelte.ts";
  import { memoryById, projectById, taskById } from "../selectors.ts";
  import { fullTimestamp, percent, timeAgo } from "../format.ts";

  let { id }: { id: string } = $props();

  let entry = $derived(app.snapshot ? memoryById(app.snapshot, id) : undefined);
  let project = $derived(entry && app.snapshot ? projectById(app.snapshot, entry.projectId) : undefined);
  let linkedTask = $derived(entry?.linkedTaskId && app.snapshot ? taskById(app.snapshot, entry.linkedTaskId) : undefined);
</script>

{#if entry}
  <Sheet title={entry.title} eyebrow={`${project?.key ?? entry.projectId} · memory`} onclose={closeSheet}>
    <div class="space-y-5">
      <div class="flex flex-wrap items-center gap-2">
        <StatusBadge status={entry.kind} />
        <span class="text-[10px] text-ink-4">updated {timeAgo(entry.updatedAt)}</span>
      </div>

      <div>
        <div class="flex items-center justify-between text-[10px] text-ink-4">
          <span>Confidence</span>
          <span class="font-mono text-ink-2">{percent(entry.confidence)}</span>
        </div>
        <div class="mt-1.5 h-1 overflow-hidden rounded-full bg-raised">
          <div class="h-full rounded-full bg-accent/70" style={`width: ${Math.round(entry.confidence * 100)}%`}></div>
        </div>
      </div>

      <p class="whitespace-pre-line text-[12px] leading-relaxed text-ink-2">{entry.content}</p>

      {#if entry.tags.length > 0}
        <Section title="Tags">
          <div class="flex flex-wrap gap-1.5">
            {#each entry.tags as tag (tag)}
              <span class="rounded border border-line-soft bg-raised px-1.5 py-0.5 font-mono text-[9.5px] text-ink-3">{tag}</span>
            {/each}
          </div>
        </Section>
      {/if}

      <Section title="Source">
        <p class="break-words rounded-control border border-line-soft bg-canvas px-3 py-2 font-mono text-[10.5px] text-ink-2">
          {entry.source}
        </p>
      </Section>

      {#if linkedTask}
        <Section title="Linked task">
          <button
            onclick={() => openTask(linkedTask!.id)}
            class="flex w-full items-center justify-between gap-2 rounded-control border border-line-soft bg-raised px-3 py-2 text-left transition-colors hover:bg-hover"
          >
            <span class="min-w-0 truncate text-[11.5px] text-ink">{linkedTask.title}</span>
            <span class="shrink-0 font-mono text-[9.5px] text-ink-4">{linkedTask.id}</span>
          </button>
        </Section>
      {/if}

      <div class="flex items-center justify-between border-t border-line-soft pt-3 text-[9.5px] text-ink-4">
        <span>created {fullTimestamp(entry.createdAt)}</span>
        <span>updated {fullTimestamp(entry.updatedAt)}</span>
      </div>
    </div>
  </Sheet>
{/if}
