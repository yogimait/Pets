<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { petForAgent } from "../pets.ts";

  let agentId = $state("");
  let pet = $derived(petForAgent(agentId));
  let pressed = false;
  let dragged = false;
  let origin = { x: 0, y: 0 };

  onMount(() => {
    const label = getCurrentWindow().label;
    agentId = label.startsWith("pet-") ? label.slice(4) : "";
    for (const element of [document.documentElement, document.body]) {
      element.style.background = "transparent";
    }
    document.body.style.overflow = "hidden";
    const app = document.getElementById("app");
    if (app) {
      app.style.height = "100%";
      app.style.background = "transparent";
    }
  });

  function showBubble() {
    if (!agentId) return;
    void invoke("show_pet_bubble", { agentId });
  }

  function hideBubble() {
    void invoke("hide_pet_bubble");
  }

  function pointerDown(event: PointerEvent) {
    if (event.button !== 0 || !agentId) return;
    pressed = true;
    dragged = false;
    origin = { x: event.clientX, y: event.clientY };
  }

  function pointerMove(event: PointerEvent) {
    if (!pressed || dragged) return;
    if (Math.hypot(event.clientX - origin.x, event.clientY - origin.y) < 4) return;
    pressed = false;
    dragged = true;
    hideBubble();
    void invoke("pet_begin_drag", { agentId });
  }

  function pointerUp() {
    if (pressed) showBubble();
    pressed = false;
    void invoke("pet_end_drag");
  }

  function pointerCancel() {
    pressed = false;
    dragged = false;
    void invoke("pet_end_drag");
  }
</script>

{#if pet}
  <button
    type="button"
    onmouseenter={showBubble}
    onmouseleave={hideBubble}
    onpointerdown={pointerDown}
    onpointermove={pointerMove}
    onpointerup={pointerUp}
    onpointercancel={pointerCancel}
    oncontextmenu={(event) => event.preventDefault()}
    aria-label={`Talk to ${pet.name}`}
    class="fixed right-0 bottom-0 block h-[80px] w-[56px] cursor-grab touch-none bg-transparent p-0 active:cursor-grabbing"
  >
    <img
      class="pet-sprite block h-[80px] w-[56px] select-none object-contain [image-rendering:pixelated]"
      src={`/pets/${pet.sprite}.png`}
      alt={pet.name}
      width="56"
      height="80"
      draggable="false"
    />
  </button>
{/if}

<style>
  @keyframes pet-bob {
    from {
      transform: translateY(0);
    }
    to {
      transform: translateY(-2px);
    }
  }

  .pet-sprite {
    animation: pet-bob 1.8s steps(2, end) infinite alternate;
  }

  @media (prefers-reduced-motion: reduce) {
    .pet-sprite {
      animation: none;
    }
  }
</style>
