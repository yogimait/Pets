<script lang="ts">
  import { onMount } from "svelte";
  import { Search } from "@lucide/svelte";
  import { app, closeSheet, go, openOrchestrator, openPets, openSheet } from "../state/app.svelte.ts";
  import { NAV, type View } from "../nav.ts";

  interface PaletteItem {
    key: string;
    label: string;
    hint: string;
    group: string;
    run: () => void;
  }

  let query = $state("");
  let index = $state(0);
  let input = $state<HTMLInputElement | null>(null);

  onMount(() => input?.focus());

  let items = $derived.by<PaletteItem[]>(() => {
    const list: PaletteItem[] = NAV.map((item) => ({
      key: `view-${item.id}`,
      label: item.label,
      hint: item.hint,
      group: "Navigate",
      run: () => go(item.id as View),
    }));
    const snapshot = app.snapshot;
    if (snapshot) {
      for (const agent of snapshot.agents) {
        list.push({
          key: `agent-${agent.id}`,
          label: agent.name,
          hint: agent.role,
          group: "Agents",
          run: () => {
            go("agents");
            openSheet("agent", agent.id);
          },
        });
      }
      for (const project of snapshot.projects) {
        list.push({
          key: `project-${project.id}`,
          label: project.name,
          hint: `${project.key} · ${project.status}`,
          group: "Projects",
          run: () => {
            go("projects");
            openSheet("project", project.id);
          },
        });
      }
      for (const task of snapshot.tasks) {
        list.push({
          key: `task-${task.id}`,
          label: task.title,
          hint: `${task.id} · ${task.status}`,
          group: "Tasks",
          run: () => {
            go("tasks");
            openSheet("task", task.id);
          },
        });
      }
    }
    list.push({
      key: "action-orchestrator",
      label: "Open orchestrator",
      hint: "Overlay · Alt+`",
      group: "Actions",
      run: () => openOrchestrator(),
    });
    list.push({
      key: "action-pets",
      label: "Open habitat",
      hint: "Pet strip",
      group: "Actions",
      run: () => openPets(),
    });
    return list;
  });

  let filtered = $derived.by(() => {
    const value = query.trim().toLowerCase();
    if (!value) return items.slice(0, 12);
    return items.filter((item) => `${item.label} ${item.hint} ${item.group}`.toLowerCase().includes(value)).slice(0, 14);
  });

  let activeIndex = $derived(Math.min(index, Math.max(filtered.length - 1, 0)));

  function close() {
    app.paletteOpen = false;
  }

  function run(item: PaletteItem | undefined) {
    if (!item) return;
    closeSheet();
    item.run();
    close();
  }

  function onKeydown(event: KeyboardEvent) {
    if (filtered.length === 0) {
      if (event.key === "Escape") close();
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      index = (activeIndex + 1) % filtered.length;
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      index = (activeIndex - 1 + filtered.length) % filtered.length;
    } else if (event.key === "Enter") {
      event.preventDefault();
      run(filtered[activeIndex]);
    } else if (event.key === "Escape") {
      close();
    }
  }
</script>

<div class="fixed inset-0 z-50">
  <button class="overlay-fade absolute inset-0 bg-black/60" aria-label="Close command palette" onclick={close}></button>
  <div
    class="relative mx-auto mt-[12vh] w-[560px] max-w-[92vw] overflow-hidden rounded-panel border border-line bg-surface"
    role="dialog"
    aria-modal="true"
    aria-label="Command palette"
  >
    <div class="flex items-center gap-2 border-b border-line-soft px-3">
      <Search size={14} strokeWidth={1.7} class="text-ink-3" />
      <input
        bind:this={input}
        bind:value={query}
        oninput={() => (index = 0)}
        onkeydown={onKeydown}
        placeholder="Jump to a view, agent, project, or task"
        aria-label="Search commands"
        class="h-10 flex-1 bg-transparent text-[12.5px] text-ink placeholder:text-ink-4 focus:outline-none"
      />
      <kbd class="rounded border border-line-soft px-1 py-0.5 font-mono text-[9.5px] text-ink-4">Esc</kbd>
    </div>

    <ul class="max-h-[340px] overflow-y-auto py-1" role="listbox" aria-label="Results">
      {#each filtered as item, i (item.key)}
        <li>
          <button
            onclick={() => run(item)}
            onmouseenter={() => (index = i)}
            class="flex w-full items-center justify-between gap-3 px-3 py-[7px] text-left transition-colors {i ===
            activeIndex
              ? 'border-l-2 border-accent bg-hover pl-[10px] text-ink'
              : 'border-l-2 border-transparent text-ink-2 hover:bg-raised/60'}"
            role="option"
            aria-selected={i === activeIndex}
          >
            <span class="truncate text-[12px]">{item.label}</span>
            <span class="shrink-0 text-[10px] text-ink-4">{item.group} · {item.hint}</span>
          </button>
        </li>
      {/each}
      {#if filtered.length === 0}
        <li class="px-3 py-6 text-center text-[11px] text-ink-3">No matches</li>
      {/if}
    </ul>
  </div>
</div>
