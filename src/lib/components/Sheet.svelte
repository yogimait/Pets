<script lang="ts">
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";
  import { X } from "@lucide/svelte";

  let {
    title,
    eyebrow,
    children,
    onclose,
  }: { title: string; eyebrow?: string; children: Snippet; onclose: () => void } = $props();

  let panel = $state<HTMLElement | null>(null);
  let closeButton = $state<HTMLButtonElement | null>(null);
  let previous: HTMLElement | null = null;

  onMount(() => {
    previous = document.activeElement as HTMLElement | null;
    closeButton?.focus();
    return () => previous?.focus?.();
  });

  function trapTab(event: KeyboardEvent) {
    if (event.key !== "Tab" || !panel) return;
    const focusable = Array.from(
      panel.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
      ),
    ).filter((element) => element.offsetParent !== null);
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }
</script>

<div class="fixed inset-0 z-40 flex justify-end">
  <button class="overlay-fade absolute inset-0 bg-black/50" aria-label="Close panel" onclick={onclose}></button>
  <div
    bind:this={panel}
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    class="sheet-panel relative flex h-full w-[460px] max-w-[92vw] flex-col border-l border-line bg-surface"
    onkeydown={trapTab}
  >
    <header class="flex items-start justify-between gap-3 border-b border-line-soft px-4 py-3">
      <div class="min-w-0">
        {#if eyebrow}<p class="truncate text-[10px] uppercase tracking-[0.12em] text-ink-3">{eyebrow}</p>{/if}
        <h2 class="mt-1 truncate text-[15px] font-medium text-ink">{title}</h2>
      </div>
      <button
        bind:this={closeButton}
        onclick={onclose}
        aria-label="Close"
        class="grid size-7 shrink-0 place-items-center rounded-control border border-line-soft text-ink-3 transition-colors hover:bg-hover hover:text-ink"
      >
        <X size={14} strokeWidth={1.8} />
      </button>
    </header>
    <div class="min-h-0 flex-1 overflow-y-auto">
      <div class="p-4">
        {@render children()}
      </div>
    </div>
  </div>
</div>
