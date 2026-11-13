<script lang="ts">
  import { onMount } from "svelte";
  import { isTauri } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import Shell from "./lib/components/Shell.svelte";
  import OrchestratorOverlay from "./lib/components/OrchestratorOverlay.svelte";
  import { app, init, navigateTo, closeOrchestrator, type SheetKind } from "./lib/state/app.svelte.ts";

  const kinds: SheetKind[] = ["agent", "project", "task", "memory", "proposal"];

  onMount(() => {
    init();
    if (!isTauri()) return;

    let cancelled = false;
    let unlisten: (() => void) | undefined;

    listen<string>("navigate", (event) => {
      const separator = event.payload.indexOf(":");
      if (separator < 0) return;
      const kind = event.payload.slice(0, separator) as SheetKind;
      const id = event.payload.slice(separator + 1);
      if (id && kinds.includes(kind)) navigateTo(kind, id);
    }).then((stop) => {
      if (cancelled) stop();
      else unlisten = stop;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  });
</script>

{#if app.ready}
  <Shell />
{:else}
  <div class="grid h-screen place-items-center text-[11.5px] text-ink-3">Loading workspace…</div>
{/if}

{#if app.overlayOpen}
  <div class="fixed inset-0 z-50 grid place-items-center bg-black/70 p-6">
    <button class="absolute inset-0" aria-label="Close orchestrator" onclick={closeOrchestrator}></button>
    <div
      class="relative h-[420px] w-[720px] max-w-full border border-[#3d3d3d] bg-[#050505]"
      role="dialog"
      aria-modal="true"
      aria-label="Orchestrator"
    >
      <OrchestratorOverlay windowMode={false} />
    </div>
  </div>
{/if}
