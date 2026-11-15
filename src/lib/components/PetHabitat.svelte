<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { X } from "@lucide/svelte";
  import { app, init } from "../state/app.svelte.ts";

  onMount(() => {
    void init();
  });

  function close() {
    void invoke("hide_pets");
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
    <span class="text-[9px] uppercase tracking-[0.16em] text-[#8a8a8a]">Habitat</span>
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
          <div
            class="group flex min-w-0 flex-col items-center gap-1 border border-[#232323] px-0.5 py-1.5"
            title={`${agent.name} · ${agent.status}${agent.currentTaskId ? ` · ${agent.currentTaskId}` : ""}`}
          >
            <span class="relative grid size-7 place-items-center border border-[#3d3d3d] bg-[#0a0a0a] text-[10px] text-[#d4d4d4]">
              {agent.name.slice(0, 2).toUpperCase()}
              <span class="absolute -right-0.5 -top-0.5 size-1.5 {tone[agent.status] ?? tone.offline}" aria-hidden="true"></span>
            </span>
            <span class="w-full truncate text-center text-[8px] uppercase tracking-[0.04em] text-[#8a8a8a]">{agent.name}</span>
          </div>
        {/each}
      </div>
    {:else}
      <span class="text-[10px] text-[#6e6e6e]">Loading habitat…</span>
    {/if}
  </div>

  <footer class="shrink-0 border-t border-[#232323] px-2.5 py-1 text-[8.5px] uppercase tracking-[0.12em] text-[#4d4d4d]">
    One habitat strip · status dots
  </footer>
</div>
