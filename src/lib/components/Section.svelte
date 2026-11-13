<script lang="ts">
  import type { Snippet } from "svelte";

  let { title, children, action }: { title: string; children: Snippet; action?: Snippet } = $props();
</script>

<section class="space-y-2">
  <div class="flex items-center justify-between gap-2">
    <h3 class="text-[9.5px] font-medium uppercase tracking-[0.14em] text-ink-4">{title}</h3>
    {#if action}{@render action()}{/if}
  </div>
  {@render children()}
</section>
