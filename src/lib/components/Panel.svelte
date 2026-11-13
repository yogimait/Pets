<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    subtitle,
    action,
    children,
    padded = true,
  }: {
    title?: string;
    subtitle?: string;
    action?: Snippet;
    children: Snippet;
    padded?: boolean;
  } = $props();
</script>

<section class="overflow-hidden rounded-panel border border-line-soft bg-surface">
  {#if title || action}
    <header class="flex h-10 items-center justify-between gap-3 border-b border-line-soft px-3.5">
      <div class="flex min-w-0 items-baseline gap-2">
        {#if title}<h2 class="shrink-0 text-[12px] font-medium text-ink">{title}</h2>{/if}
        {#if subtitle}<span class="truncate text-[10.5px] text-ink-3">{subtitle}</span>{/if}
      </div>
      {#if action}{@render action()}{/if}
    </header>
  {/if}
  <div class={padded ? "p-3.5" : ""}>
    {@render children()}
  </div>
</section>
