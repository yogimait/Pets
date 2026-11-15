<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { petForAgent } from "../pets.ts";

  let agentId = $state("");
  let pet = $derived(petForAgent(agentId));

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
</script>

{#if pet}
  <button
    type="button"
    onmouseenter={showBubble}
    onmouseleave={hideBubble}
    onclick={showBubble}
    aria-label={`Talk to ${pet.name}`}
    class="grid h-screen w-screen place-items-end bg-transparent p-0"
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
