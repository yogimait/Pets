<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { LogicalSize } from "@tauri-apps/api/dpi";
  import { ChevronDown, X } from "@lucide/svelte";
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

  // Collapsed mode: the pen shrinks to a draggable pill that reopens on click.
  const COLLAPSED_WIDTH = 132;
  const COLLAPSED_HEIGHT = 44;
  const EXPANDED_MIN: [number, number] = [320, 220];
  let collapsed = $state(false);
  let downX = 0;
  let downY = 0;

  let runningCount = $derived(
    app.snapshot?.tasks.filter((task) => task.status === "running").length ?? 0,
  );

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

  function applyCollapsed(next: boolean) {
    const win = getCurrentWindow();
    if (next) {
      try {
        const width = window.innerWidth;
        const height = window.innerHeight;
        if (width > COLLAPSED_WIDTH && height > COLLAPSED_HEIGHT) {
          localStorage.setItem("petpen-size", `${width}x${height}`);
        }
      } catch {
        /* keep the last saved size */
      }
      win.setMinSize(new LogicalSize(COLLAPSED_WIDTH, COLLAPSED_HEIGHT));
      win.setSize(new LogicalSize(COLLAPSED_WIDTH, COLLAPSED_HEIGHT));
    } else {
      win.setMinSize(new LogicalSize(EXPANDED_MIN[0], EXPANDED_MIN[1]));
      const saved = localStorage.getItem("petpen-size") ?? "600x420";
      const [rawW, rawH] = saved.split("x").map(Number);
      const width = Number.isFinite(rawW) && rawW >= EXPANDED_MIN[0] ? rawW : 600;
      const height = Number.isFinite(rawH) && rawH >= EXPANDED_MIN[1] ? rawH : 420;
      win.setSize(new LogicalSize(width, height));
    }
    localStorage.setItem("petpen-collapsed", next ? "1" : "0");
    collapsed = next;
  }

  // Click-vs-drag for the header and the collapsed pill. The native window
  // drag must NOT start on pointerdown: once it runs it swallows the mouse
  // up, so a real click would never toggle the collapse. Start the drag only
  // after the pointer moves; a still press toggles instead.
  let pressPointerId: number | null = null;
  let pressDragging = false;

  function penPressDown(event: PointerEvent) {
    if (event.button !== 0) return;
    if ((event.target as HTMLElement).closest("button")) return;
    downX = event.clientX;
    downY = event.clientY;
    pressDragging = false;
    pressPointerId = event.pointerId;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function penPressMove(event: PointerEvent) {
    if (pressPointerId !== event.pointerId) return;
    if (!pressDragging && Math.hypot(event.clientX - downX, event.clientY - downY) > 6) {
      pressDragging = true;
      void getCurrentWindow().startDragging();
    }
  }

  function penPressUp(event: PointerEvent) {
    if (pressPointerId !== event.pointerId) return;
    pressPointerId = null;
    if (pressDragging) return;
    void applyCollapsed(!collapsed);
  }

  function penPressCancel() {
    pressPointerId = null;
    pressDragging = false;
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
    if (localStorage.getItem("petpen-collapsed") === "1") {
      void applyCollapsed(true);
    }
    last = performance.now();
    frame = requestAnimationFrame(tick);
  });

  onDestroy(() => {
    mounted = false;
    cancelAnimationFrame(frame);
    if (hideTimer) clearTimeout(hideTimer);
  });
</script>

{#if collapsed}
  <div
    role="button"
    tabindex="0"
    aria-label="Open pet pen"
    title="Click to open the pet pen"
    onpointerdown={penPressDown}
    onpointermove={penPressMove}
    onpointerup={penPressUp}
    onpointercancel={penPressCancel}
    onlostpointercapture={penPressCancel}
    onkeydown={(event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        void applyCollapsed(false);
      }
    }}
    class="group grid h-full w-full cursor-move place-items-center select-none"
  >
    <span
      class="flex items-center gap-2 rounded-full border border-white/25 bg-[#0a0b0d]/85 px-4 py-2 shadow-[0_4px_14px_rgba(0,0,0,0.5)] transition-colors group-hover:border-white/45"
    >
      <span class="text-[15px] leading-none" aria-hidden="true">🐾</span>
      <span class="font-mono text-[10px] uppercase tracking-[0.16em] text-white/70">Pets</span>
      {#if runningCount > 0}
        <span
          class="grid size-4 place-items-center rounded-full border border-white/25 bg-[#39d353] font-mono text-[9px] font-semibold text-black"
          aria-label={`${runningCount} running`}
        >{runningCount}</span>
      {/if}
      <ChevronDown size={12} strokeWidth={2.4} class="text-white/50" aria-hidden="true" />
    </span>
  </div>
{:else}
<div class="relative h-screen w-screen overflow-hidden rounded-lg border border-white/20 bg-[#0a0b0d]/25 text-ink shadow-[0_0_0_1px_rgba(0,0,0,0.4)]">
  <header
    role="toolbar"
    tabindex="0"
    aria-label="Pet pen window controls"
    title="Click to collapse the pen"
    onpointerdown={penPressDown}
    onpointermove={penPressMove}
    onpointerup={penPressUp}
    onpointercancel={penPressCancel}
    onlostpointercapture={penPressCancel}
    onkeydown={(event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        void applyCollapsed(true);
      }
    }}
    class="flex h-8 cursor-move select-none items-center justify-between border-b border-white/10 bg-[#0a0b0d]/70 px-2.5"
  >
    <span class="font-mono text-[9px] uppercase tracking-[0.16em] text-white/55">Pet pen</span>
    <span class="flex items-center gap-1.5 font-mono text-[9px] text-white/35">
      click header to collapse · drag to move · corner to resize
      <ChevronDown size={11} strokeWidth={2.4} class="-rotate-180 text-white/50" aria-hidden="true" />
    </span>
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
{/if}

<style>
  :global(html),
  :global(body),
  :global(#app) {
    background: transparent !important;
    overflow: hidden;
  }
</style>
