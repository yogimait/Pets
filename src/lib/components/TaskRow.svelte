<script lang="ts">
  import type { Snapshot, Task } from "../types.ts";
  import { agentById } from "../selectors.ts";
  import { openTask } from "../state/app.svelte.ts";
  import StatusBadge from "./StatusBadge.svelte";

  let { task, snapshot }: { task: Task; snapshot: Snapshot } = $props();

  let owner = $derived(agentById(snapshot, task.ownerId));
</script>

<button
  onclick={() => openTask(task.id)}
  class="flex w-full items-center gap-3 border-b border-line-soft px-3.5 py-2.5 text-left transition-colors last:border-b-0 hover:bg-raised/50"
>
  <StatusBadge status={task.status} />
  <span class="min-w-0 flex-1 truncate text-[12px] text-ink">{task.title}</span>
  <span class="hidden shrink-0 text-[10.5px] text-ink-3 sm:block">{owner?.name ?? task.ownerId}</span>
  <span class="shrink-0 font-mono text-[10px] text-ink-4">{task.id}</span>
</button>
