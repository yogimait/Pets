<script lang="ts">
  import Sheet from "./Sheet.svelte";
  import { app, createTask } from "../state/app.svelte.ts";
  import type { TaskPriority } from "../types.ts";

  let { onclose }: { onclose: () => void } = $props();

  const specialists = $derived(
    (app.snapshot?.agents ?? []).filter((agent) => agent.id !== "orchestrator" && agent.id !== "dreamer"),
  );
  const projects = $derived(app.snapshot?.projects ?? []);

  let projectId = $state("core");
  let ownerId = $state("pixel");
  let title = $state("");
  let description = $state("");
  let acceptance = $state("");
  let priority = $state<TaskPriority>("normal");
  let autoStart = $state(true);
  let requiredMcp = $state<string[]>([]);
  let busy = $state(false);
  let error = $state("");

  const mcpOptions = [
    { id: "playwright", label: "Playwright" },
    { id: "n8n-mcp", label: "n8n" },
    { id: "stitch", label: "Stitch" },
    { id: "rea", label: "REA" },
    { id: "jev", label: "Jev" },
    { id: "laya", label: "Laya" },
  ];

  const inputClass =
    "h-8 w-full rounded-control border border-line-soft bg-canvas px-2.5 text-[11.5px] text-ink placeholder:text-ink-4 focus:border-accent/40 focus:outline-none";
  const areaClass =
    "w-full resize-y rounded-control border border-line-soft bg-canvas px-2.5 py-2 text-[11.5px] leading-relaxed text-ink placeholder:text-ink-4 focus:border-accent/40 focus:outline-none";

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (busy || !title.trim() || !description.trim()) return;
    busy = true;
    error = "";
    try {
      const saved = await createTask({
        projectId,
        title: title.trim(),
        description: description.trim(),
        ownerId,
        priority,
        acceptance: acceptance.trim(),
        autoStart,
        requiredMcp,
      });
      if (saved) onclose();
      else if (app.workforceError) error = app.workforceError;
    } finally {
      busy = false;
    }
  }
</script>

<Sheet title="New task" eyebrow="Manual task" {onclose}>
  <form class="space-y-4" onsubmit={submit}>
    <div class="grid grid-cols-2 gap-2">
      <label class="space-y-1">
        <span class="block text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Project</span>
        <select bind:value={projectId} class={inputClass}>
          {#each projects as project (project.id)}<option value={project.id}>{project.key} · {project.name}</option>{/each}
        </select>
      </label>
      <label class="space-y-1">
        <span class="block text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Owner</span>
        <select bind:value={ownerId} class={inputClass}>
          {#each specialists as agent (agent.id)}<option value={agent.id}>{agent.name}</option>{/each}
        </select>
      </label>
    </div>

    <label class="block space-y-1">
      <span class="block text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Title</span>
      <input bind:value={title} class={inputClass} maxlength="120" placeholder="What should be done?" />
    </label>

    <label class="block space-y-1">
      <span class="block text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Description</span>
      <textarea
        bind:value={description}
        class={areaClass}
        rows="4"
        maxlength="4000"
        placeholder="One deliverable, with the boundary of the work."
      ></textarea>
    </label>

    <label class="block space-y-1">
      <span class="block text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Acceptance</span>
      <textarea
        bind:value={acceptance}
        class={areaClass}
        rows="3"
        placeholder="How the result is verified."
      ></textarea>
    </label>

    <fieldset class="space-y-1.5">
      <legend class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Required MCP · optional</legend>
      <div class="flex flex-wrap gap-x-3 gap-y-1.5">
        {#each mcpOptions as option (option.id)}
          <label class="flex items-center gap-1.5 text-[10.5px] text-ink-3">
            <input bind:group={requiredMcp} value={option.id} type="checkbox" class="size-3.5 accent-[#8ca6c0]" />
            {option.label}
          </label>
        {/each}
      </div>
      <p class="text-[9.5px] leading-relaxed text-ink-4">The task blocks before starting if a selected server is failed or still starting.</p>
    </fieldset>

    <div class="flex items-center gap-2">
      <label class="flex items-center gap-2 text-[11px] text-ink-2">
        <span class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Priority</span>
        <select bind:value={priority} class="h-8 rounded-control border border-line-soft bg-canvas px-2 text-[11px] text-ink-2">
          <option value="low">Low</option>
          <option value="normal">Normal</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </label>
      <label class="ml-auto flex items-center gap-2 text-[11px] text-ink-2">
        <input bind:checked={autoStart} type="checkbox" class="size-3.5 accent-[#8ca6c0]" />
        Start immediately
      </label>
    </div>

    {#if error}
      <p class="rounded-control border border-bad/30 bg-bad/5 px-2.5 py-2 text-[10.5px] leading-relaxed text-bad" role="alert">
        {error}
      </p>
    {/if}

    <div class="flex justify-end gap-2 border-t border-line-soft pt-3">
      <button
        type="button"
        onclick={onclose}
        class="rounded-control border border-line-soft px-3 py-1.5 text-[11px] text-ink-3 transition-colors hover:bg-hover hover:text-ink"
      >
        Cancel
      </button>
      <button
        type="submit"
        disabled={busy || !title.trim() || !description.trim()}
        class="rounded-control border border-accent/40 bg-accent-soft px-3 py-1.5 text-[11px] text-ink transition-colors hover:border-accent/60 disabled:opacity-40"
      >
        {busy ? "Creating…" : "Create task"}
      </button>
    </div>
  </form>
</Sheet>
