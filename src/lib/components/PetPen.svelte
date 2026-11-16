<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { X } from "@lucide/svelte";
  import { app, init } from "../state/app.svelte.ts";
  import { PETS, type PetMeta } from "../pets.ts";

  type PetState = PetMeta & {
    x: number;
    y: number;
    targetX: number;
    targetY: number;
    nextTarget: number;
    dragging: boolean;
  };

  const HOME: [number, number][] = [
    [0.5, 0.72],
    [0.14, 0.76],
    [0.86, 0.72],
    [0.7, 0.68],
    [0.3, 0.72],
    [0.6, 0.82],
    [0.4, 0.66],
    [0.52, 0.84],
    [0.9, 0.78],
  ];
  const WORK: [number, number][] = [
    [0.5, 0.5],
    [0.18, 0.56],
    [0.8, 0.5],
    [0.64, 0.46],
    [0.26, 0.5],
    [0.56, 0.58],
    [0.4, 0.44],
    [0.48, 0.62],
    [0.84, 0.54],
  ];

  let pets = $state<PetState[]>(
    PETS.map((pet, index) => ({
      ...pet,
      x: HOME[index][0],
      y: HOME[index][1],
      targetX: HOME[index][0],
      targetY: HOME[index][1],
      nextTarget: 0,
      dragging: false,
    })),
  );
  let selected = $state<string | null>(null);
  let pinned = $state(false);
  let hideTimer: ReturnType<typeof setTimeout> | undefined;
  let frame = 0;
  let last = 0;
  let mounted = false;

  let selectedPet = $derived(pets.find((pet) => pet.agentId === selected));
  let selectedAgent = $derived(
    app.snapshot?.agents.find((agent) => agent.id === selected) ?? null,
  );
  let selectedTask = $derived(
    app.snapshot?.tasks.find(
      (task) =>
        task.id === selectedAgent?.currentTaskId ||
        (task.ownerId === selected && task.status === "running"),
    ) ?? null,
  );
  let selectedActivity = $derived(
    app.snapshot?.activity.find((entry) => entry.actorId === selected) ?? null,
  );

  function showBubble(agentId: string, pin = false) {
    if (hideTimer) clearTimeout(hideTimer);
    selected = agentId;
    pinned = pin;
    if (!pin) hideTimer = setTimeout(hideBubble, 4_000);
  }

  function hideBubble() {
    if (pinned) return;
    selected = null;
  }

  function closeBubble() {
    pinned = false;
    if (hideTimer) clearTimeout(hideTimer);
    selected = null;
  }

  function startWindowDrag(event: PointerEvent) {
    if ((event.target as HTMLElement).closest("button")) return;
    void getCurrentWindow().startDragging();
  }

  function startResize(event: PointerEvent) {
    event.preventDefault();
    event.stopPropagation();
    void getCurrentWindow().startResizeDragging("SouthEast");
  }

  function startPetDrag(event: PointerEvent, pet: PetState) {
    if (event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    pet.dragging = true;
    pet.nextTarget = Number.POSITIVE_INFINITY;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function movePet(event: PointerEvent, pet: PetState) {
    if (!pet.dragging) return;
    const width = Math.max(window.innerWidth - 56, 1);
    const height = Math.max(window.innerHeight - 80, 1);
    pet.x = Math.min(1, Math.max(0, (event.clientX - 28) / width));
    pet.y = Math.min(1, Math.max(0, (event.clientY - 40) / height));
    pet.targetX = pet.x;
    pet.targetY = pet.y;
  }

  function endPetDrag(pet: PetState) {
    if (!pet.dragging) return;
    pet.dragging = false;
    pet.nextTarget = performance.now() + 2_500;
  }

  function tick(now: number) {
    if (!mounted) return;
    const dt = Math.min((now - last) / 1000, 0.1);
    last = now;
    for (const [index, pet] of pets.entries()) {
      if (pet.dragging) continue;
      const agent = app.snapshot?.agents.find((candidate) => candidate.id === pet.agentId);
      const working =
        agent?.status === "active" || Boolean(agent?.currentTaskId);
      const base = working ? WORK[index] : HOME[index];
      if (now >= pet.nextTarget) {
        const spread = working ? 0.16 : 0.08;
        const noise = (seed: number) => {
          const value = Math.sin(seed * 12.9898 + index * 78.233) * 43758.5453;
          return value - Math.floor(value);
        };
        pet.targetX = Math.min(0.92, Math.max(0.08, base[0] + (noise(now * 0.001) - 0.5) * spread));
        pet.targetY = Math.min(0.84, Math.max(0.16, base[1] + (noise(now * 0.001 + 7) - 0.5) * spread));
        pet.nextTarget = now + (working ? 1_400 : 3_400);
      }
      if (!working && Math.floor(now / 1000 + index) % 11 === 0) continue;
      const dx = pet.targetX - pet.x;
      const dy = pet.targetY - pet.y;
      const distance = Math.hypot(dx, dy);
      const speed = (working ? 0.13 : 0.045) * dt;
      if (distance > 0.001) {
        pet.x += (dx / distance) * Math.min(speed, distance);
        pet.y += (dy / distance) * Math.min(speed, distance);
      }
    }
    frame = requestAnimationFrame(tick);
  }

  onMount(() => {
    mounted = true;
    for (const element of [document.documentElement, document.body]) {
      element.style.background = "transparent";
    }
    document.body.style.overflow = "hidden";
    const root = document.getElementById("app");
    if (root) {
      root.style.height = "100%";
      root.style.background = "transparent";
    }
    void init();
    last = performance.now();
    frame = requestAnimationFrame(tick);
  });

  onDestroy(() => {
    mounted = false;
    cancelAnimationFrame(frame);
    if (hideTimer) clearTimeout(hideTimer);
  });
</script>

<div class="relative h-screen w-screen overflow-hidden rounded-lg border border-white/20 bg-[#0a0b0d]/25 text-ink shadow-[0_0_0_1px_rgba(0,0,0,0.4)]">
  <header
    role="toolbar"
    tabindex="-1"
    aria-label="Pet pen window controls"
    onpointerdown={startWindowDrag}
    class="flex h-8 cursor-move select-none items-center justify-between border-b border-white/10 bg-[#0a0b0d]/70 px-2.5"
  >
    <span class="font-mono text-[9px] uppercase tracking-[0.16em] text-white/55">Pet pen</span>
    <span class="font-mono text-[9px] text-white/35">drag to move · corner to resize</span>
  </header>

  <div class="relative h-[calc(100%-2rem)] w-full">
    {#if selectedPet && selectedAgent}
      <div
        role="dialog"
        tabindex="-1"
        aria-label={`${selectedPet.name} activity`}
        class="absolute left-1/2 top-2 z-20 w-[300px] -translate-x-1/2 border-2 border-[#111] bg-[#f5f0e6] p-2.5 text-[#111] shadow-[4px_4px_0_#111]"
        onmouseenter={() => hideTimer && clearTimeout(hideTimer)}
        onmouseleave={hideBubble}
      >
        <div class="flex items-start justify-between gap-2">
          <div>
            <p class="text-[12px] font-bold uppercase tracking-[0.08em]">{selectedPet.name}</p>
            <p class="text-[9px] uppercase tracking-[0.12em] text-[#5b554b]">{selectedPet.role}</p>
          </div>
          <button
            type="button"
            onclick={closeBubble}
            aria-label="Close speech bubble"
            class="grid size-5 place-items-center border border-[#111] text-[#111] hover:bg-[#111] hover:text-[#f5f0e6]"
          >
            <X size={11} strokeWidth={2.4} />
          </button>
        </div>
        <p class="mt-2 text-[9.5px] font-semibold uppercase tracking-[0.12em]">{selectedAgent.status}</p>
        <p class="mt-1 line-clamp-2 text-[10.5px] leading-snug">
          {selectedTask ? selectedTask.title : "No active task."}
        </p>
        {#if selectedActivity}
          <p class="mt-1 line-clamp-2 border-t border-[#b9b1a3] pt-1 text-[9px] leading-snug text-[#5b554b]">
            {selectedActivity.summary}
          </p>
        {/if}
        <span class="absolute -bottom-[9px] left-1/2 -ml-[9px] h-0 w-0 border-l-[9px] border-r-[9px] border-t-[10px] border-l-transparent border-r-transparent border-t-[#111]"></span>
      </div>
    {/if}

    {#each pets as pet (pet.agentId)}
      {@const agent = app.snapshot?.agents.find((candidate) => candidate.id === pet.agentId)}
      <button
        type="button"
        aria-label={`${pet.name} · ${pet.role} · ${agent?.status ?? "idle"}`}
        class="absolute z-10 grid h-20 w-14 cursor-grab touch-none place-items-end bg-transparent p-0 active:cursor-grabbing"
        style={`left: calc(${pet.x * 100}% - 28px); top: calc(${pet.y * 100}% - 40px);`}
        onmouseenter={() => showBubble(pet.agentId)}
        onmouseleave={hideBubble}
        onclick={() => showBubble(pet.agentId, true)}
        onpointerdown={(event) => startPetDrag(event, pet)}
        onpointermove={(event) => movePet(event, pet)}
        onpointerup={() => endPetDrag(pet)}
        onpointercancel={() => endPetDrag(pet)}
      >
        <img
          src={`/pets/${pet.sprite}.png`}
          alt={pet.name}
          width="56"
          height="80"
          draggable="false"
          class="pointer-events-none block h-20 w-14 select-none object-contain [image-rendering:pixelated]"
        />
      </button>
    {/each}
  </div>

  <button
    type="button"
    aria-label="Resize pet pen"
    onpointerdown={startResize}
    class="absolute bottom-0 right-0 z-30 size-5 cursor-nwse-resize border-b-2 border-r-2 border-white/30"
  ></button>
</div>

<style>
  :global(html),
  :global(body),
  :global(#app) {
    background: transparent !important;
    overflow: hidden;
  }
</style>
