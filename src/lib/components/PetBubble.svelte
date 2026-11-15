<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { X } from "@lucide/svelte";
  import { app, init } from "../state/app.svelte.ts";
  import { petForAgent } from "../pets.ts";

  let agentId = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;

  let pet = $derived(petForAgent(agentId));
  let agent = $derived(app.snapshot?.agents.find((candidate) => candidate.id === agentId) ?? null);
  let currentTask = $derived(
    app.snapshot?.tasks.find(
      (task) => task.id === agent?.currentTaskId || (task.ownerId === agentId && task.status === "running"),
    ),
  );
  let latest = $derived(app.snapshot?.activity.find((entry) => entry.actorId === agentId) ?? null);

  function scheduleHide() {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => void invoke("hide_pet_bubble"), 4_000);
  }

  onMount(() => {
    for (const element of [document.documentElement, document.body]) {
      element.style.background = "transparent";
    }
    document.body.style.overflow = "hidden";
    const root = document.getElementById("app");
    if (root) root.style.background = "transparent";
    void init();
    const interval = setInterval(async () => {
      try {
        const next = await invoke<string | null>("pet_bubble_agent");
        if (next !== agentId) {
          agentId = next;
          if (next) scheduleHide();
        }
      } catch {
        // Bubble is optional; the pet window remains usable if this fails.
      }
    }, 400);
    return () => {
      if (timer) clearTimeout(timer);
      clearInterval(interval);
    };
  });

  function hide() {
    if (timer) clearTimeout(timer);
    void invoke("hide_pet_bubble");
  }
</script>

{#if pet}
  <div
    role="dialog"
    tabindex="-1"
    aria-label={`${pet.name} activity`}
    class="relative w-[320px] border-2 border-[#111] bg-[#f5f0e6] p-3 text-[#111] shadow-[4px_4px_0_#111]"
    onmouseenter={() => timer && clearTimeout(timer)}
    onmouseleave={scheduleHide}
  >
    <div class="flex items-start justify-between gap-2">
      <div>
        <p class="text-[13px] font-bold uppercase tracking-[0.08em]">{pet.name}</p>
        <p class="text-[9px] uppercase tracking-[0.12em] text-[#5b554b]">{pet.role}</p>
      </div>
      <button
        onclick={hide}
        aria-label="Close speech bubble"
        class="grid size-5 place-items-center border border-[#111] text-[#111] transition-colors hover:bg-[#111] hover:text-[#f5f0e6]"
      >
        <X size={11} strokeWidth={2.4} />
      </button>
    </div>

    <p class="mt-2 text-[10px] font-semibold uppercase tracking-[0.12em]">{agent?.status ?? "idle"}</p>
    <p class="mt-1 line-clamp-2 text-[11px] leading-snug">
      {currentTask ? currentTask.title : "No active task."}
    </p>
    {#if latest}
      <p class="mt-1 line-clamp-2 border-t border-[#b9b1a3] pt-1 text-[9px] leading-snug text-[#5b554b]">
        {latest.summary}
      </p>
    {/if}
    <span class="absolute -bottom-[9px] left-1/2 -ml-[9px] h-0 w-0 border-l-[9px] border-r-[9px] border-t-[10px] border-l-transparent border-r-transparent border-t-[#111]"></span>
  </div>
{/if}

<style>
  :global(html),
  :global(body),
  :global(#app) {
    background: transparent !important;
    overflow: hidden;
  }
</style>
