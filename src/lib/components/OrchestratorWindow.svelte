<script lang="ts">
  import { onMount } from "svelte";
  import OrchestratorOverlay from "./OrchestratorOverlay.svelte";
  import { app, init } from "../state/app.svelte.ts";

  onMount(() => {
    init();
  });
</script>

{#if app.ready}
  <OrchestratorOverlay windowMode />
{:else}
  <div class="grid h-screen place-items-center bg-[#050505] text-[11px] text-[#6e6e6e]">Loading orchestrator…</div>
{/if}
