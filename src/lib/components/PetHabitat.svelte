<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { X } from "@lucide/svelte";
  import { app, init } from "../state/app.svelte.ts";
  import type { Agent } from "../types.ts";

  onMount(() => {
    void init();
  });

  let active = $state<Agent | null>(null);

  function close() {
    void invoke("hide_pets");
  }

  function openAgent(id: string) {
    void invoke("open_record", { kind: "agent", id });
  }

  const tone: Record<string, string> = {
    active: "bg-[#39d353]",
    idle: "bg-[#4d4d4d]",
    paused: "bg-[#c8a05a]",
    offline: "bg-[#c07a7a]",
  };
</script>

<div class="flex h-screen flex-col overflow-hidden border border-[#3d3d3d] bg-[#050505] text-[#ededed]">
  <header class="flex h-7 shrink-0 items-center justify-between border-b border-[#232323] px-2.5">
    <span data-tauri-drag-region class="flex-1 text-[9px] uppercase tracking-[0.16em] text-[#8a8a8a]">Habitat</span>
    <button
      onclick={close}
      aria-label="Close habitat"
      class="grid size-6 place-items-center text-[#6e6e6e] transition-colors hover:text-[#ededed]"
    >
      <X size={12} strokeWidth={2} />
    </button>
  </header>

  <div class="flex min-h-0 flex-1 items-center px-2.5">
    {#if app.snapshot}
      <div class="grid w-full grid-cols-9 gap-1.5">
        {#each app.snapshot.agents.filter((agent) => agent.id !== "orchestrator" && agent.id !== "dreamer") as agent (agent.id)}
          <button
            type="button"
            onmouseenter={() => (active = agent)}
            onmouseleave={() => (active = null)}
            onfocus={() => (active = agent)}
            onblur={() => (active = null)}
            onclick={() => openAgent(agent.id)}
            aria-label={`Inspect ${agent.name}, ${agent.status}`}
            title={`${agent.name} · ${agent.status}${agent.currentTaskId ? ` · ${agent.currentTaskId}` : ""}`}
            class="group flex min-w-0 flex-col items-center gap-1 border border-[#232323] px-0.5 py-1.5 transition-colors hover:border-[#3d3d3d] focus:border-[#ededed] focus:outline-none"
          >
            <span class="relative grid size-7 place-items-center border border-[#3d3d3d] bg-[#0a0a0a] text-[10px] text-[#d4d4d4]">
              {agent.name.slice(0, 2).toUpperCase()}
              <span class="absolute -right-0.5 -top-0.5 size-1.5 {tone[agent.status] ?? tone.offline}" aria-hidden="true"></span>
            </span>
            <span class="w-full truncate text-center text-[8px] uppercase tracking-[0.04em] text-[#8a8a8a]">{agent.name}</span>
          </button>
        {/each}
      </div>
    {:else}
      <span class="text-[10px] text-[#6e6e6e]">Loading habitat…</span>
    {/if}
  </div>

  <footer data-tauri-drag-region class="shrink-0 border-t border-[#232323] px-2.5 py-1 text-[8.5px] uppercase tracking-[0.12em] text-[#4d4d4d]">
    {#if active}
      <span class="text-[#8a8a8a]">{active.name}</span> · {active.role} · {active.status}{active.currentTaskId
        ? ` · ${active.currentTaskId}`
        : ""}
    {:else}
      Hover a specialist · click to inspect
    {/if}
  </footer>
</div>
