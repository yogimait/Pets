<script lang="ts">
  import type { ActivityEvent, Snapshot } from "../types.ts";
  import { agentById, projectById } from "../selectors.ts";
  import { clockTime } from "../format.ts";
  import { openTask } from "../state/app.svelte.ts";
  import Avatar from "./Avatar.svelte";

  let { event, snapshot, showProject = true }: { event: ActivityEvent; snapshot: Snapshot; showProject?: boolean } = $props();

  let actor = $derived(agentById(snapshot, event.actorId));
  let project = $derived(showProject ? projectById(snapshot, event.projectId) : undefined);

  const rowClass =
    "flex w-full gap-2.5 border-b border-line-soft px-3.5 py-2.5 text-left transition-colors last:border-b-0";
</script>

{#snippet content()}
  <Avatar name={actor?.name ?? "System"} size="xs" />
  <div class="min-w-0 flex-1">
    <p class="text-[11.5px] leading-snug text-ink-2">
      <span class="text-ink">{actor?.name ?? event.actorId}</span>
      {event.summary}
    </p>
    <div class="mt-1 flex flex-wrap items-center gap-2 text-[9.5px] text-ink-4">
      <span class="font-mono">{clockTime(event.at)}</span>
      {#if project}<span class="text-ink-3">{project.key}</span>{/if}
      <span>{event.kind}</span>
    </div>
  </div>
{/snippet}

{#if event.taskId}
  <button onclick={() => openTask(event.taskId!)} class="{rowClass} hover:bg-raised/50">
    {@render content()}
  </button>
{:else}
  <div class={rowClass}>
    {@render content()}
  </div>
{/if}
